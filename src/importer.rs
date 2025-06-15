//! # Terraform Import Module
//! 
//! This module handles the core logic for importing existing cloud resources into Terraform state
//! using Terragrunt. It processes Terraform plan files, maps resources to modules, infers resource IDs,
//! and generates or executes terragrunt import commands.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use crate::plan::TerraformResource;
use crate::reporting::{ImportStats, ImportOperation, print_import_progress, print_import_summary};
use crate::utils::collect_resources;
use crate::schema::SchemaManager;

/// Represents a resource that has been processed and has an inferred ID
/// 
/// This structure contains all the information needed to execute a terragrunt import
/// command for a specific resource, including the resource metadata, the inferred ID,
/// and the module information.
#[derive(Debug)]
pub struct ResourceWithId<'a> {
    /// Reference to the original resource from the plan file
    pub resource: &'a Resource,
    /// Terraform resource representation for internal processing
    pub terraform_resource: TerraformResource,
    /// The inferred ID that will be used for the import command
    pub id: String,
    /// Metadata about the module this resource belongs to
    pub module_meta: &'a ModuleMeta,
    /// File system path to the module directory
    pub module_path: PathBuf,
}

/// Result of processing a single resource
/// 
/// Represents the outcome when attempting to process a resource for import.
/// A resource can either be ready for import or skipped for various reasons.
#[derive(Debug)]
enum ResourceProcessingResult<'a> {
    /// Resource is ready for import with all required information
    ReadyForImport(ResourceWithId<'a>),
    /// Resource was skipped with an explanation
    Skipped { 
        /// The resource address that was skipped
        address: String, 
        /// Human-readable reason why the resource was skipped
        reason: String 
    },
}

/// Result of executing an import operation
/// 
/// Represents the outcome of attempting to execute a terragrunt import command.
#[derive(Debug)]
enum ImportExecutionResult {
    /// Import command executed successfully
    Success(String), // resource address
    /// Import command failed with error details
    Failed { 
        /// The resource address that failed
        address: String, 
        /// Error message describing the failure
        error: String 
    },
    /// Dry run mode - command was generated but not executed
    DryRun { 
        /// The resource address for the dry run
        address: String, 
        /// The command string that would be executed
        command: String 
    },
}

/// Represents a Terraform plan file structure
/// 
/// This is the top-level structure when parsing a Terraform plan JSON file.
/// Contains all the information about planned changes, provider schemas, and variables.
#[derive(Debug, Deserialize)]
pub struct PlanFile {
    /// Format version of the plan file
    pub format_version: String,
    /// Version of Terraform that generated this plan
    pub terraform_version: String,
    /// Input variables used in the plan
    pub variables: Option<Variables>,
    /// The planned values (resources to be created/modified)
    pub planned_values: Option<PlannedValues>,
    /// Provider schema information
    pub provider_schemas: Option<ProviderSchemas>,
}

/// Provider schema information from a plan file
/// 
/// Contains the schema definitions for all resource types supported by a provider.
#[derive(Debug, Deserialize)]
pub struct ProviderSchema {
    /// Map of resource type names to their schema definitions
    pub resource_schemas: Option<HashMap<String, serde_json::Value>>,
}

/// Collection of all provider schemas in the plan
/// 
/// Maps provider names to their schema definitions.
#[derive(Debug, Deserialize)]
pub struct ProviderSchemas {
    /// Map of provider names to their schema information
    pub provider_schemas: HashMap<String, ProviderSchema>,
}

/// Input variables from the Terraform plan
/// 
/// Contains commonly used variables like project_id and region.
#[derive(Debug, Deserialize)]
pub struct Variables {
    /// GCP project ID variable
    pub project_id: Option<ValueWrapper>,
    /// Cloud region variable
    pub region: Option<ValueWrapper>,
}

/// Wrapper for variable values in the plan file
/// 
/// Terraform plan files wrap variable values in this structure.
#[derive(Debug, Deserialize)]
pub struct ValueWrapper {
    /// The actual variable value
    pub value: String,
}

/// Planned values section of a Terraform plan
/// 
/// Contains the root module with all planned resources and child modules.
#[derive(Debug, Deserialize)]
pub struct PlannedValues {
    /// The root module containing all planned resources
    pub root_module: PlannedModule,
}

/// Represents a Terraform module in the plan
/// 
/// A module can contain resources and child modules. This structure is used
/// recursively to represent the full module hierarchy.
#[derive(Debug, Deserialize)]
pub struct PlannedModule {
    /// Resources directly contained in this module
    pub resources: Option<Vec<Resource>>,
    /// Child modules nested within this module
    pub child_modules: Option<Vec<PlannedModule>>,
    /// Address/path of this module (e.g., "module.vpc")
    pub address: Option<String>,
}

/// Represents a single Terraform resource in the plan
/// 
/// Contains all the metadata and configuration for a planned resource.
#[derive(Debug, Deserialize)]
pub struct Resource {
    /// Full resource address (e.g., "module.vpc.aws_vpc.main")
    pub address: String,
    /// Resource mode - typically "managed" for regular resources
    pub mode: String,
    /// Resource type (e.g., "aws_vpc", "google_storage_bucket")
    #[serde(rename = "type")]
    pub r#type: String,
    /// Resource name within the module
    pub name: String,
    /// Provider that manages this resource
    pub provider_name: Option<String>,
    /// Schema version for this resource type
    pub schema_version: Option<u32>,
    /// Resource configuration values
    pub values: Option<serde_json::Value>,
    /// Sensitive values that are redacted in output
    pub sensitive_values: Option<serde_json::Value>,
    /// Resources this resource depends on
    pub depends_on: Option<Vec<String>>,
}

/// Metadata about a Terragrunt module
/// 
/// This comes from the modules.json file generated by terragrunt and contains
/// information about where each module is located and what it contains.
#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleMeta {
    /// Unique key identifying this module
    #[serde(rename = "Key")]
    pub key: String,
    /// Source path or URL for this module
    #[serde(rename = "Source")]
    pub source: String,
    /// Directory path where this module is located
    #[serde(rename = "Dir")]
    pub dir: String,
}

/// Collection of all module metadata
/// 
/// Top-level structure from the modules.json file.
#[derive(Debug, Deserialize, Serialize)]
pub struct ModulesFile {
    /// Array of all modules in the workspace
    #[serde(rename = "Modules")]
    pub modules: Vec<ModuleMeta>,
}

/// **TEST UTILITY FUNCTION** - Validates that module directories exist on the filesystem
/// 
/// This function is primarily used in tests to verify that the module structure
/// is valid and all referenced directories actually exist.
/// 
/// # Arguments
/// * `modules` - Array of module metadata to validate
/// * `module_root` - Root directory to resolve relative paths against
/// 
/// # Returns
/// A vector of error messages for any missing or invalid directories
pub fn validate_module_dirs<P: AsRef<Path>>(
    modules: &[ModuleMeta],
    module_root: P,
) -> Vec<String> {
    modules
        .iter()
        .filter_map(|module| {
            let path = module_root.as_ref().join(&module.dir);
            if !path.is_dir() {
                Some(format!("Missing or invalid directory: {}", path.display()))
            } else {
                None
            }
        })
        .collect()
}

/// Maps resources from a plan file to their corresponding module metadata
/// 
/// This function creates a mapping between resource addresses and the modules
/// that contain them, enabling us to know which module directory to use when
/// running terragrunt import commands.
/// 
/// # Arguments
/// * `modules` - Array of module metadata from modules.json
/// * `plan` - Parsed Terraform plan file
/// 
/// # Returns
/// HashMap mapping resource addresses to their module metadata
pub fn map_resources_to_modules<'a>(
    modules: &'a [ModuleMeta],
    plan: &'a PlanFile,
) -> HashMap<String, &'a ModuleMeta> {
    let mut mapping = HashMap::new();

    if let Some(planned_values) = &plan.planned_values {
        /// Recursively processes modules to build the resource-to-module mapping
        fn recurse_modules<'a>(
            modules: &'a [ModuleMeta],
            module: &'a PlannedModule,
            mapping: &mut HashMap<String, &'a ModuleMeta>,
        ) {
            if let Some(resources) = &module.resources {
                if let Some(address) = &module.address {
                    let prefix = address.strip_prefix("module.").unwrap_or("");
                    if let Some(module_meta) = modules.iter().find(|m| m.key == prefix) {
                        for resource in resources {
                            mapping.insert(resource.address.clone(), module_meta);
                        }
                    } else {
                        eprintln!("⚠️ Warning: Unmatched module address '{}' - skipping resources in this module", address);
                        // Continue processing instead of crashing
                    }
                }
            }
            if let Some(children) = &module.child_modules {
                for child in children {
                    recurse_modules(modules, child, mapping);
                }
            }
        }

        recurse_modules(modules, &planned_values.root_module, &mut mapping);
    }

    mapping
}

/// **TEST UTILITY FUNCTION** - Generates terragrunt import command strings
/// 
/// This function is primarily used in tests to verify command generation logic.
/// For production use, prefer `execute_or_print_imports` which handles the full workflow.
/// 
/// # Arguments
/// * `resource_map` - Mapping of resources to their modules
/// * `plan` - Terraform plan file
/// * `module_root` - Root directory for resolving module paths
/// * `verbose` - Whether to print verbose output during processing
/// 
/// # Returns
/// Vector of terragrunt import command strings
pub fn generate_import_commands(
    resource_map: &HashMap<String, &ModuleMeta>,
    plan: &PlanFile,
    module_root: &str,
    verbose: bool,
) -> Vec<String> {
    let mut commands = vec![];

    if let Some(planned_values) = &plan.planned_values {
        let mut all_resources = vec![];
        collect_resources(&planned_values.root_module, &mut all_resources);

        for resource in all_resources {
            let terraform_resource = TerraformResource {
                address: resource.address.clone(),
                mode: resource.mode.clone(),
                r#type: resource.r#type.clone(),
                name: resource.name.clone(),
                values: resource.values.clone(),
            };

            let _schema_map = plan
                .provider_schemas
                .as_ref()
                .and_then(|ps| ps.provider_schemas.values().next())
                .and_then(|provider| provider.resource_schemas.as_ref())
                .cloned()
                .unwrap_or_default();

            let inferred_id = infer_resource_id(
                &terraform_resource,
                None, // Schema manager not available in test utility (use main workflow instead)
                verbose,
            );

            if let Some(module_meta) = resource_map.get(&resource.address) {
                if let Some(ref id) = inferred_id {
                    let full_path = PathBuf::from(module_root).join(&module_meta.dir);
                    let cmd = format!(
                        "terragrunt import -config-dir={} {} {}",
                        full_path.display(),
                        resource.address,
                        id
                    );
                    commands.push(cmd);
                } else if verbose {
                    println!(
                        "⚠️ Could not infer ID for resource {}",
                        resource.address
                    );
                }
            }
        }
    }

    commands
}

/// **TEST UTILITY FUNCTION** - Fallback ID candidate extraction from resource values
/// 
/// This function provides a basic fallback for extracting potential ID fields when no
/// provider schema is available. It's primarily used in tests and as a last resort.
/// Prefer using SchemaManager for production use.
/// 
/// # Arguments
/// * `values` - Map of resource attribute values
/// 
/// # Returns
/// Set of attribute names that could potentially be used as resource IDs
pub fn extract_id_candidate_fields_from_values(values: &Map<String, Value>) -> HashSet<String> {
    let mut fields = HashSet::new();

    for (key, val) in values.iter() {
        if val.is_string() || val.is_number() || val.is_boolean() {
            fields.insert(key.clone());
        }
    }

    fields
}

/// Infers the most likely ID for a resource based on its attributes
/// 
/// This function analyzes a resource's attributes to determine the best candidate
/// for use as an import ID. It uses schema-driven intelligence when available,
/// falling back to hardcoded heuristics for backward compatibility.
/// 
/// # Arguments
/// * `resource` - The terraform resource to analyze
/// * `schema_manager` - Optional schema manager for schema-driven intelligent analysis
/// * `verbose` - Whether to print debug information about the selection process
/// 
/// # Returns
/// The inferred ID string if one could be determined, None otherwise
/// 
/// # Schema-Driven Intelligence
/// When a SchemaManager is provided, this function uses intelligent scoring based on:
/// - **Attribute Metadata**: Required, computed, optional fields from terraform schema
/// - **Type Analysis**: String types preferred for human-readable IDs
/// - **Resource-Specific Logic**: Special handling for known resource types
/// - **Score-Based Ranking**: Attributes ranked by calculated relevance scores
/// 
/// # Fallback Behavior
/// Without a SchemaManager, falls back to hardcoded priority order for backward compatibility.
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::importer::infer_resource_id;
/// use terragrunt_import_from_plan::plan::TerraformResource;
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// use serde_json::json;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let resource = TerraformResource {
///     address: "google_storage_bucket.example".to_string(),
///     mode: "managed".to_string(),
///     r#type: "google_storage_bucket".to_string(),
///     name: "example".to_string(),
///     values: Some(json!({"name": "my-bucket", "location": "us-central1"})),
/// };
/// 
/// // Schema-driven approach (preferred)
/// let mut schema_manager = SchemaManager::new("./envs/gcp/dev");
/// schema_manager.load_or_generate_schema()?;
/// let id = infer_resource_id(&resource, Some(&schema_manager), true);
/// 
/// // Fallback approach
/// let id_fallback = infer_resource_id(&resource, None, false);
/// # Ok(())
/// # }
/// ```
pub fn infer_resource_id(
    resource: &TerraformResource,
    schema_manager: Option<&SchemaManager>,
    verbose: bool,
) -> Option<String> {
    let values = resource.values.as_ref()?.as_object()?;
    
    if let Some(manager) = schema_manager {
        // ✨ NEW: Use schema-driven intelligent scoring
        match manager.get_id_candidate_attributes(&resource.r#type) {
            Ok(candidates) => {
                if verbose {
                    println!("🧠 [{}] Using schema-driven ID inference", resource.address);
                    for (name, metadata) in candidates.iter().take(5) {
                        println!("  📊 {} (score: {:.1}, required: {}, computed: {})", 
                                name, metadata.calculate_base_score(), metadata.required, metadata.computed);
                    }
                }
                
                // Try candidates in score order (highest first)
                for (attr_name, _metadata) in candidates {
                    if let Some(val) = values.get(&attr_name) {
                        if let Some(s) = val.as_str() {
                            if verbose {
                                println!("✅ [{}] Selected schema-driven ID: {} = '{}'", 
                                        resource.address, attr_name, s);
                            }
                            return Some(s.to_string());
                        }
                    }
                }
                
                if verbose {
                    println!("⚠️ [{}] Schema-driven candidates found but no string values available", resource.address);
                }
            }
            Err(e) => {
                if verbose {
                    println!("⚠️ [{}] Schema analysis failed: {}, falling back to hardcoded approach", resource.address, e);
                }
            }
        }
    }
    
    // Fallback to hardcoded approach (for backward compatibility or when schema fails)
    let priority_order = vec!["id", "name", "bucket", "self_link", "project"];
    let mut ranked_candidates = priority_order
        .iter()
        .filter_map(|&field| values.get(field).map(|v| (field, v)))
        .collect::<Vec<_>>();

    // Add remaining fields that aren't in priority order
    for (key, val) in values {
        if !priority_order.contains(&key.as_str()) {
            ranked_candidates.push((key, val));
        }
    }

    if verbose && schema_manager.is_none() {
        println!("🔍 [{}] Using fallback hardcoded approach. Candidates: {:?}",
                resource.address,
                ranked_candidates.iter().map(|(k, _)| *k).collect::<Vec<_>>()
        );
    }

    for (field_name, val) in ranked_candidates {
        if let Some(s) = val.as_str() {
            if verbose {
                println!("✅ [{}] Selected fallback ID: {} = '{}'", 
                        resource.address, field_name, s);
            }
            return Some(s.to_string());
        }
    }

    if verbose {
        println!("❌ [{}] No suitable ID field found", resource.address);
    }
    None
}

/// Executes an import operation for a resource (either dry-run or real execution)
/// 
/// This internal function handles the actual execution or simulation of a terragrunt
/// import command for a single resource.
/// 
/// # Arguments
/// * `resource_with_id` - Resource with all information needed for import
/// * `dry_run` - If true, only generate the command without executing it
/// 
/// # Returns
/// Result indicating success, failure, or dry-run status
fn execute_import_for_resource(resource_with_id: &ResourceWithId, dry_run: bool) -> ImportExecutionResult {
    if dry_run {
        let command = format!(
            "terragrunt import -config-dir={} {} {}",
            resource_with_id.module_path.display(),
            resource_with_id.resource.address,
            resource_with_id.id
        );
        ImportExecutionResult::DryRun {
            address: resource_with_id.resource.address.clone(),
            command,
        }
    } else {
        match run_terragrunt_import(&resource_with_id.module_path, &resource_with_id.resource.address, resource_with_id.id.clone()) {
            Ok(_) => ImportExecutionResult::Success(resource_with_id.resource.address.clone()),
            Err(e) => ImportExecutionResult::Failed {
                address: resource_with_id.resource.address.clone(),
                error: format!("Import failed: {}", e),
            },
        }
    }
}

/// Processes a single resource and determines if it's ready for import or should be skipped
/// 
/// This internal function analyzes a resource to determine if it can be imported.
/// It checks for module mapping, attempts ID inference using schema-driven intelligence,
/// and returns the appropriate result.
/// 
/// # Arguments
/// * `resource` - The resource to process
/// * `resource_map` - Mapping of resources to modules
/// * `schema_manager` - Schema manager for intelligent ID inference
/// * `module_root` - Root directory for module paths
/// * `verbose` - Whether to print verbose output
/// 
/// # Returns
/// Processing result indicating if the resource is ready for import or should be skipped
fn process_single_resource<'a>(
    resource: &'a Resource,
    resource_map: &HashMap<String, &'a ModuleMeta>,
    schema_manager: Option<&SchemaManager>,
    module_root: &str,
    verbose: bool,
) -> ResourceProcessingResult<'a> {
    let terraform_resource = TerraformResource {
        address: resource.address.clone(),
        mode: resource.mode.clone(),
        r#type: resource.r#type.clone(),
        name: resource.name.clone(),
        values: resource.values.clone(),
    };

    let inferred_id = infer_resource_id(
        &terraform_resource,
        schema_manager, // ✅ Now uses schema-driven intelligence
        verbose,
    );

    match resource_map.get(&resource.address) {
        Some(module_meta) => {
            if let Some(id) = inferred_id {
                let module_path = PathBuf::from(module_root).join(&module_meta.dir);
                ResourceProcessingResult::ReadyForImport(ResourceWithId {
                    resource,
                    terraform_resource,
                    id,
                    module_meta,
                    module_path,
                })
            } else {
                ResourceProcessingResult::Skipped {
                    address: resource.address.clone(),
                    reason: "no ID could be inferred".to_string(),
                }
            }
        }
        None => ResourceProcessingResult::Skipped {
            address: resource.address.clone(),
            reason: "no matching module mapping found".to_string(),
        },
    }
}

/// Collects all resources from a plan and prepares the provider schema map
/// 
/// This internal helper function extracts all resources from a plan file and
/// prepares the provider schema information for use in processing.
/// 
/// # Arguments
/// * `plan` - The Terraform plan file to process
/// 
/// # Returns
/// Tuple of (all resources, provider schema map)
fn collect_and_prepare_resources(plan: &PlanFile) -> (Vec<&Resource>, HashMap<String, Value>) {
    let mut all_resources = vec![];
    if let Some(planned_values) = &plan.planned_values {
        collect_resources(&planned_values.root_module, &mut all_resources);
    }

    let schema_map = plan
        .provider_schemas
        .as_ref()
        .and_then(|ps| ps.provider_schemas.values().next())
        .and_then(|provider| provider.resource_schemas.as_ref())
        .cloned()
        .unwrap_or_default();

    (all_resources, schema_map)
}

/// **MAIN FUNCTION** - Executes or prints terragrunt import commands for all resources
/// 
/// This is the primary function used by the application to process a full Terraform plan
/// and either execute import commands or print them in dry-run mode. It handles the complete
/// workflow from resource discovery to import execution/simulation with schema-driven intelligence.
/// 
/// # Arguments
/// * `resource_map` - Mapping of resource addresses to their module metadata
/// * `plan` - Terraform plan file containing resources to import
/// * `dry_run` - If true, only print commands without executing them
/// * `verbose` - Whether to print detailed progress information
/// * `module_root` - Root directory for resolving module paths
/// * `working_directory` - Directory containing terragrunt configuration for schema loading
/// 
/// # Schema-Driven Intelligence
/// This function attempts to load provider schema for intelligent ID inference. If schema
/// loading fails, it gracefully falls back to hardcoded heuristics for backward compatibility.
pub fn execute_or_print_imports(
    resource_map: &HashMap<String, &ModuleMeta>,
    plan: &PlanFile,
    dry_run: bool,
    verbose: bool,
    module_root: &str,
    working_directory: Option<&str>,
) {
    if let Some(_planned_values) = &plan.planned_values {
        let (all_resources, _schema_map) = collect_and_prepare_resources(plan);
        let mut stats = ImportStats::new();

        // ✨ NEW: Attempt to load schema for intelligent ID inference
        let schema_manager = if let Some(work_dir) = working_directory {
            let mut manager = SchemaManager::new(work_dir);
            match manager.load_or_generate_schema() {
                Ok(_) => {
                    if verbose {
                        println!("✅ Loaded provider schema for intelligent ID inference");
                    }
                    Some(manager)
                }
                Err(e) => {
                    if verbose {
                        println!("⚠️ Schema loading failed, using fallback approach: {}", e);
                    }
                    None
                }
            }
        } else {
            if verbose {
                println!("⚠️ No working directory provided, using fallback ID inference");
            }
            None
        };

        for resource in all_resources {
            if verbose {
                print_import_progress(&resource.address, ImportOperation::Checking);
            }

            let result = process_single_resource(resource, resource_map, schema_manager.as_ref(), module_root, verbose);

            match result {
                ResourceProcessingResult::ReadyForImport(resource_with_id) => {
                    if verbose {
                        print_import_progress(&resource_with_id.resource.address, ImportOperation::Importing { id: resource_with_id.id.clone() });
                    }
                    
                    let execution_result = execute_import_for_resource(&resource_with_id, dry_run);

                    match execution_result {
                        ImportExecutionResult::Success(address) => {
                            print_import_progress(&address, ImportOperation::Success);
                            stats.increment_imported(address.clone());
                        }
                        ImportExecutionResult::Failed { address, error } => {
                            print_import_progress(&address, ImportOperation::Failed { error });
                            stats.increment_failed();
                        }
                        ImportExecutionResult::DryRun { address, command } => {
                            print_import_progress(&address, ImportOperation::DryRun { command });
                            stats.increment_imported(address.clone());
                        }
                    }
                }
                ResourceProcessingResult::Skipped { address, reason } => {
                    print_import_progress(&address, ImportOperation::Skipped { reason });
                    stats.increment_skipped();
                }
            }
        }

        print_import_summary(&stats);
    }
}

/// Executes a terragrunt import command for a single resource
/// 
/// This function runs the actual `terragrunt import` command in the specified module
/// directory. It handles error cases and provides detailed error messages.
/// 
/// # Arguments
/// * `module_path` - Directory containing the terragrunt module
/// * `resource_address` - Full resource address (e.g., "aws_vpc.main")
/// * `resource_id` - Cloud resource ID to import
/// 
/// # Returns
/// Result indicating success or failure with error details
/// 
/// # Errors
/// - Returns error if module path doesn't exist
/// - Returns error if terragrunt command fails
/// - Captures both stdout and stderr for debugging
pub fn run_terragrunt_import(
    module_path: &Path,
    resource_address: &str,
    resource_id: String,
) -> io::Result<()> {
    if !module_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Module path does not exist: {}", module_path.display()),
        ));
    }

    let output = Command::new("terragrunt")
        .arg("import")
        .arg(resource_address)
        .arg(resource_id)
        .current_dir(module_path)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_output = if !stderr.trim().is_empty() {
            stderr.trim()
        } else if !stdout.trim().is_empty() {
            stdout.trim()
        } else {
            "No error output captured"
        };
        
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to import {} (exit code: {}): {}", 
                resource_address, 
                output.status.code().unwrap_or(-1),
                error_output
            ),
        ))
    }
}

/// **INTERNAL HELPER** - Recursively checks modules for resources with inferable IDs
/// 
/// This is an internal helper function used for debugging and testing. It recursively
/// walks through a module hierarchy looking for resources that can have IDs inferred.
/// 
/// # Arguments
/// * `module` - The module to check
/// * `found` - Mutable reference to track if any IDs were found
/// * `verbose` - Whether to print verbose output
/// * `_schema_map` - Schema information (currently unused)
fn check(
    module: &PlannedModule,
    found: &mut bool,
    verbose: bool,
    _schema_map: &HashMap<String, Value>,
) {
    if let Some(resources) = &module.resources {
        for resource in resources {
            let terraform_resource = TerraformResource {
                address: resource.address.clone(),
                mode: resource.mode.clone(),
                r#type: resource.r#type.clone(),
                name: resource.name.clone(),
                values: resource.values.clone(),
            };

            if let Some(id) = infer_resource_id(&terraform_resource, None, verbose) {
                println!("Inferred ID for {}: {}", terraform_resource.address, id);
                *found = true;
                return;
            }
        }
    }

    if let Some(children) = &module.child_modules {
        for child in children {
            check(child, found, verbose, _schema_map);
        }
    }
}

