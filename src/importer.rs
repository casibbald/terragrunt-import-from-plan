use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{self};
use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::{Map, Value};
use crate::plan::TerraformResource;
use crate::reporting::{ImportStats, ImportOperation, print_import_progress, print_import_summary};
use crate::utils::collect_resources;
use crate::schema::SchemaManager;

/// Represents a resource that has been processed and has an inferred ID
#[derive(Debug)]
pub struct ResourceWithId<'a> {
    pub resource: &'a Resource,
    pub terraform_resource: TerraformResource,
    pub id: String,
    pub module_meta: &'a ModuleMeta,
    pub module_path: PathBuf,
}

/// Result of processing a single resource
#[derive(Debug)]
enum ResourceProcessingResult<'a> {
    ReadyForImport(ResourceWithId<'a>),
    Skipped { address: String, reason: String },
}

/// Result of executing an import operation
#[derive(Debug)]
enum ImportExecutionResult {
    Success(String), // address
    Failed { address: String, error: String },
    DryRun { address: String, command: String },
}

#[derive(Debug, Deserialize)]
pub struct PlanFile {
    pub format_version: String,
    pub terraform_version: String,
    pub variables: Option<Variables>,
    pub planned_values: Option<PlannedValues>,
    pub provider_schemas: Option<ProviderSchemas>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderSchema {
    pub resource_schemas: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderSchemas {
    pub provider_schemas: HashMap<String, ProviderSchema>,
}

#[derive(Debug, Deserialize)]
pub struct Variables {
    pub project_id: Option<ValueWrapper>,
    pub region: Option<ValueWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct ValueWrapper {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct PlannedValues {
    pub root_module: PlannedModule,
}

#[derive(Debug, Deserialize)]
pub struct PlannedModule {
    pub resources: Option<Vec<Resource>>,
    pub child_modules: Option<Vec<PlannedModule>>,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Resource {
    pub address: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub name: String,
    pub provider_name: Option<String>,
    pub schema_version: Option<u32>,
    pub values: Option<serde_json::Value>,
    pub sensitive_values: Option<serde_json::Value>,
    pub depends_on: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleMeta {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Source")]
    pub source: String,
    #[serde(rename = "Dir")]
    pub dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModulesFile {
    #[serde(rename = "Modules")]
    pub modules: Vec<ModuleMeta>,
}



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


pub fn map_resources_to_modules<'a>(
    modules: &'a [ModuleMeta],
    plan: &'a PlanFile,
) -> HashMap<String, &'a ModuleMeta> {
    let mut mapping = HashMap::new();

    if let Some(planned_values) = &plan.planned_values {
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
                None, // Will be updated when we integrate SchemaManager properly
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


/// Fallback when no provider schema is available.
/// Scores likely identifier fields based on naming heuristics.
pub fn extract_id_candidate_fields_from_values(values: &Map<String, Value>) -> HashSet<String> {
    let mut fields = HashSet::new();

    for (key, val) in values.iter() {
        if val.is_string() || val.is_number() || val.is_boolean() {
            fields.insert(key.clone());
        }
    }

    fields
}

pub fn infer_resource_id(
    resource: &TerraformResource,
    schema_manager: Option<&SchemaManager>,
    verbose: bool,
) -> Option<String> {
    let values = resource.values.as_ref()?.as_object()?;
    
    let candidates = if let Some(manager) = schema_manager {
        manager.extract_id_candidates(&resource.r#type)
    } else {
        // fallback for test cases without schema manager
        SchemaManager::extract_id_candidates_from_values(values)
    };

    // Always prioritize these if present
    let priority_order = vec!["id", "name", "bucket", "self_link", "project"];
    let mut ranked_candidates = priority_order
        .iter()
        .filter_map(|&field| values.get(field).map(|v| (field, v)))
        .collect::<Vec<_>>();

    // Then add remaining fields from the schema
    for (key, val) in values {
        if candidates.contains(key) && !priority_order.contains(&key.as_str()) {
            ranked_candidates.push((key, val));
        }
    }

    if verbose {
        println!(
            "🔍 [{}] Ranked ID candidates: {:?}",
            resource.address,
            ranked_candidates.iter().map(|(k, _)| *k).collect::<Vec<_>>()
        );
    }

    for (_, val) in ranked_candidates {
        if let Some(s) = val.as_str() {
            return Some(s.to_string());
        }
    }

    None
}

/// Executes an import operation for a resource (either dry-run or real execution)
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
fn process_single_resource<'a>(
    resource: &'a Resource,
    resource_map: &HashMap<String, &'a ModuleMeta>,
    _schema_map: &HashMap<String, Value>,
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
        None, // Will be updated when we integrate SchemaManager properly
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

pub fn execute_or_print_imports(
    resource_map: &HashMap<String, &ModuleMeta>,
    plan: &PlanFile,
    dry_run: bool,
    verbose: bool,
    module_root: &str,
) {
    if let Some(_planned_values) = &plan.planned_values {
        let (all_resources, schema_map) = collect_and_prepare_resources(plan);
        let mut stats = ImportStats::new();

        for resource in all_resources {
            if verbose {
                print_import_progress(&resource.address, ImportOperation::Checking);
            }

            let result = process_single_resource(resource, resource_map, &schema_map, module_root, verbose);

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

