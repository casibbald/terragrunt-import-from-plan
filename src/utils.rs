//! # Utility Functions Module
//! 
//! This module provides utility functions for various terragrunt and terraform operations,
//! including workspace management, fixture generation, validation, and command execution.
//! 
//! ## Key Functionality
//! 
//! - **Resource Collection**: Recursive collection of resources from terraform modules
//! - **Schema Processing**: Extraction of ID candidate fields from provider schemas
//! - **Workspace Management**: Cleaning and setup of terragrunt workspaces
//! - **Fixture Generation**: Creation of test fixtures for multiple cloud providers
//! - **Validation**: Terraform formatting and configuration validation
//! - **Module Operations**: Init, plan, apply, and destroy operations for terragrunt modules
//! 
//! ## Error Handling
//! 
//! Most functions return `anyhow::Result` for comprehensive error reporting.
//! Some functions support "safe mode" which continues execution despite failures.

use crate::importer::{PlannedModule, Resource};
use serde_json::Value;
use std::collections::HashSet;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

pub use crate::schema::write_provider_schema;

/// Extracts and displays terraform plan summaries from command output
/// 
/// Parses terraform/terragrunt output to find and display plan summaries
/// such as "Plan: X to add, Y to change, Z to destroy" or "No changes".
/// 
/// # Arguments
/// * `output` - The command output to parse (stdout or stderr)
/// 
/// # Returns
/// Optional string containing the extracted plan summary
fn extract_plan_summary(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        
        // Look for various plan summary patterns
        if line.starts_with("Plan: ") && (line.contains("to add") || line.contains("to change") || line.contains("to destroy")) {
            return Some(line.to_string());
        }
        
        // Handle "No changes" case
        if line.starts_with("No changes.") && line.contains("infrastructure matches") {
            return Some(line.to_string());
        }
        
        // Handle apply/destroy summaries too
        if line.starts_with("Apply complete!") || line.starts_with("Destroy complete!") {
            return Some(line.to_string());
        }
        
        // Handle warnings about changes outside terraform
        if line.contains("Warning:") && line.contains("outside of Terraform") {
            return Some(line.to_string());
        }
    }
    
    None
}

/// Error types for terragrunt process execution
/// 
/// Represents various failure modes when executing terragrunt commands,
/// providing detailed context about what went wrong.
#[derive(Error, Debug)]
pub enum TerragruntProcessError {
    /// Process execution failed with specific exit status and output
    #[error("Failed to run terragrunt: status={status}, stdout={stdout}, stderr={stderr}")]
    ProcessError {
        /// Exit status code from the process
        status: i32,
        /// Standard output from the command
        stdout: String,
        /// Standard error output from the command
        stderr: String,
    },
    /// I/O error occurred during process execution
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Collects all resources from a planned module and its child modules recursively
/// 
/// This function traverses the module hierarchy and collects all resources into
/// a flat vector. It handles both direct resources in the module and resources
/// in nested child modules.
/// 
/// # Arguments
/// * `module` - The planned module to collect resources from
/// * `resources` - Mutable vector to append collected resources to
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::collect_resources;
/// use terragrunt_import_from_plan::importer::{PlannedModule, Resource};
/// 
/// let root_module = PlannedModule {
///     resources: Some(vec![]),
///     child_modules: None,
///     address: None,
/// };
/// let mut all_resources = Vec::new();
/// collect_resources(&root_module, &mut all_resources);
/// println!("Found {} resources", all_resources.len());
/// ```
pub fn collect_resources<'a>(module: &'a PlannedModule, resources: &mut Vec<&'a Resource>) {
    if let Some(module_resources) = &module.resources {
        resources.extend(module_resources.iter());
    }
    if let Some(children) = &module.child_modules {
        for child in children {
            collect_resources(child, resources);
        }
    }
}

/// Extracts potential ID candidate field names from provider schema JSON
/// 
/// Analyzes provider schema information to identify attribute names that could
/// potentially be used as resource identifiers. This provides a fallback when
/// more sophisticated schema analysis isn't available.
/// 
/// # Arguments
/// * `schema_json` - JSON containing provider schema information
/// 
/// # Returns
/// Set of attribute names that appear in resource schemas
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::extract_id_candidate_fields;
/// use serde_json::json;
/// 
/// let schema_json = json!({
///     "provider_schemas": {
///         "registry.terraform.io/hashicorp/aws": {
///             "resource_schemas": {
///                 "aws_instance": {
///                     "block": {
///                         "attributes": {
///                             "id": {},
///                             "name": {}
///                         }
///                     }
///                 }
///             }
///         }
///     }
/// });
/// let candidates = extract_id_candidate_fields(&schema_json);
/// println!("Found {} potential ID fields", candidates.len());
/// ```
pub fn extract_id_candidate_fields(schema_json: &Value) -> HashSet<String> {
    let mut candidates = HashSet::new();

    if let Some(provider_schemas) = schema_json
        .get("provider_schemas")
        .and_then(|ps| ps.as_object())
    {
        // Iterate through all providers (aws, google, azurerm, etc.)
        for (_provider_name, provider_schema) in provider_schemas {
            if let Some(resource_schemas) = provider_schema
                .get("resource_schemas")
                .and_then(|rs| rs.as_object())
            {
                for (_resource_type, schema) in resource_schemas {
                    if let Some(block) = schema.get("block") {
                        if let Some(attributes) = block.get("attributes").and_then(|a| a.as_object()) {
                            for (attr_name, _) in attributes {
                                candidates.insert(attr_name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    candidates
}

/// Initializes a terragrunt workspace by running `terragrunt init`
/// 
/// Executes terragrunt initialization in the specified directory, which downloads
/// providers, modules, and sets up the workspace for subsequent operations.
/// 
/// # Arguments
/// * `working_directory` - Directory containing terragrunt configuration
/// 
/// # Returns
/// Result indicating success or failure with detailed error information
/// 
/// # Errors
/// - Command execution failure
/// - Non-zero exit status from terragrunt init
/// - I/O errors accessing the directory
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::run_terragrunt_init;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// run_terragrunt_init("./envs/dev")?;
/// # Ok(())
/// # }
/// ```
pub fn run_terragrunt_init(working_directory: &str) -> Result<()> {
    println!("🔧 Running `terragrunt init` in {}", working_directory);

    let output = Command::new("terragrunt")
        .arg("init")
        .current_dir(working_directory)
        .output()
        .with_context(|| format!("Failed to execute terragrunt init in directory: {}", working_directory))?;

    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Terragrunt init failed in {}\nStatus: {}\nStdout: {}\nStderr: {}",
            working_directory,
            output.status.code().unwrap_or(-1),
            stdout.trim(),
            stderr.trim()
        );
    }
}

/// Cleans up terragrunt workspace files and directories
/// 
/// Removes various terragrunt and terraform artifacts including .terragrunt-cache,
/// .terraform directories, state files, lock files, and provider-specific files.
/// Can target a specific provider or clean all providers.
/// 
/// # Arguments
/// * `provider` - Optional provider name to clean specifically, or None to clean all
/// * `deep_clean` - If true, also removes test fixtures (generated artifacts)
/// 
/// # Returns
/// Result indicating success or failure
/// 
 /// # Files/Directories Removed
/// - `.terraform` directories (recursive)
/// - `.terragrunt-cache` directories (recursive)  
/// - `*.tfstate` files (recursive)
/// - `*.lock.hcl` files (recursive)
/// - `*.tfplan` files (recursive, all terraform plan files)
/// - Provider workspace files: `plan.json`, `out.json`, schema files
/// - Simulator directory artifacts (per provider)
/// - Test fixtures: `out.json`, `modules.json` (only with deep_clean=true)
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::clean_workspace;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Regular clean - preserves test fixtures
/// clean_workspace(None, false)?;
/// 
/// // Deep clean - removes everything including test fixtures
/// clean_workspace(Some("aws"), true)?;
/// # Ok(())
/// # }
/// ```
pub fn clean_workspace(provider: Option<&str>, deep_clean: bool) -> Result<()> {
    /// Recursively removes directories with the specified name
    fn remove_dirs_by_name(root: &Path, dir_name: &str) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().map(|n| n == dir_name).unwrap_or(false) {
                        let _ = fs::remove_dir_all(&path);
                    } else {
                        remove_dirs_by_name(&path, dir_name);
                    }
                }
            }
        }
    }

    /// Recursively removes files with the specified extension
    fn remove_files_by_ext(root: &Path, ext: &str) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    remove_files_by_ext(&path, ext);
                } else if path.extension().map(|e| e == ext).unwrap_or(false) {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }

    /// Recursively removes files ending with the specified pattern
    fn remove_files_by_pattern(root: &Path, pattern: &str) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    remove_files_by_pattern(&path, pattern);
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(pattern) {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }

    /// Recursively removes files with the exact specified name
    fn remove_files_by_name(root: &Path, file_name: &str) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    remove_files_by_name(&path, file_name);
                } else if path.file_name().map(|n| n == file_name).unwrap_or(false) {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }

    let root = Path::new(".");
    remove_dirs_by_name(root, ".terraform");
    remove_dirs_by_name(root, ".terragrunt-cache");
    remove_files_by_ext(root, "tfstate");
    remove_files_by_pattern(root, ".lock.hcl");
    remove_files_by_ext(root, "tfplan"); // Remove ALL .tfplan files by extension

    // Clean provider-specific files for all providers or specific provider
    let providers = if let Some(p) = provider {
        vec![p]
    } else {
        vec!["aws", "gcp", "azure"]
    };

    for provider_name in providers {
        let env_path = Path::new("envs/simulator").join(provider_name).join("dev");
        if env_path.exists() {
            remove_files_by_name(&env_path, "plan.json");
            remove_files_by_name(&env_path, "out.json"); // Clean workspace out.json files
            remove_files_by_name(&env_path, ".terragrunt-provider-schema.json");
        }
        
        // Also clean the simulator directory for this provider
        let simulator_path = Path::new("simulator").join(provider_name);
        if simulator_path.exists() {
            remove_files_by_ext(&simulator_path, "tfplan"); // Remove any .tfplan files in simulator
            remove_files_by_name(&simulator_path, "plan.json");
            remove_files_by_name(&simulator_path, "out.json");
        }
        
        // Clean dynamically generated test fixtures (these are artifacts, not source files)
        // Only remove when doing a deep clean to avoid breaking integration tests
        if deep_clean {
            let fixtures_path = Path::new("tests/fixtures").join(provider_name);
            if fixtures_path.exists() {
                remove_files_by_name(&fixtures_path, "out.json"); // Generated by generate_plan_json()
                remove_files_by_name(&fixtures_path, "modules.json"); // Generated by generate_modules_json()
            }
        }
    }

    Ok(())
}

/// Creates a minimal plan JSON file when terraform planning fails
/// 
/// This is an internal helper function that creates a basic plan.json file
/// with minimal structure when the actual terraform plan operation fails.
/// This ensures tests and other operations can continue with a fallback.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// 
/// # Returns
/// Result indicating success or failure of file creation
fn create_minimal_plan_json(provider: &str) -> Result<()> {
    let minimal_plan = r#"{"format_version":"1.2","terraform_version":"1.9.8","planned_values":{"root_module":{"child_modules":[]}}}"#;
    let fixtures_dir = format!("tests/fixtures/{}", provider);
    fs::create_dir_all(&fixtures_dir)?;
    fs::write(format!("{}/out.json", fixtures_dir), minimal_plan)?;
    println!("⚠️ Created minimal out.json for {} provider (plan failed)", provider);
    Ok(())
}

/// Generates test fixtures for a specific cloud provider
/// 
/// This function performs a complete workflow to generate test fixtures including:
/// 1. Cleaning the workspace
/// 2. Running terragrunt init
/// 3. Generating provider schema
/// 4. Running terragrunt plan
/// 5. Generating modules.json
/// 6. Converting plan files to JSON format
/// 
/// The function is designed to be fault-tolerant and will create minimal fixtures
/// even if some steps fail (e.g., in CI environments without cloud credentials).
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// 
/// # Returns
/// Result indicating success or failure of fixture generation
/// 
/// # Errors
/// - Invalid provider name
/// - Missing environment directory
/// - File system errors during fixture creation
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::generate_fixtures;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// generate_fixtures("aws")?;
/// generate_fixtures("gcp")?;
/// generate_fixtures("azure")?;
/// # Ok(())
/// # }
/// ```
pub fn generate_fixtures(provider: &str) -> Result<()> {
    println!("🔧 Generating fixtures for {} provider...", provider);
    
    // Clean workspace for this provider
    clean_workspace(Some(provider), false)?;

    let env_path = format!("envs/simulator/{}/dev", provider);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    // Init
    println!("🚀 Running terragrunt init for {}...", provider);
    let init_output = Command::new("terragrunt")
        .arg("init")
        .arg("--all")
        .current_dir(&env_path)
        .env("TF_IN_AUTOMATION", "true")
        .env("CHECKPOINT_DISABLE", "true")
        .output()
        .with_context(|| format!("Failed to run terragrunt init for {}", provider))?;

    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        eprintln!("⚠️ Warning: terragrunt init failed for {}: {}", provider, stderr);
        // Continue despite init failure (expected in CI)
    }

    // Generate provider schema using our built-in functionality
    println!("📋 Generating provider schema for {}...", provider);
    if let Err(e) = write_provider_schema(Path::new(&env_path)) {
        eprintln!("⚠️ Warning: provider schema generation failed for {}: {}", provider, e);
        // Continue despite schema generation failure (expected in CI without credentials)
    } else {
        println!("✅ Provider schema generated successfully for {}", provider);
    }

    // Plan
    println!("📋 Running terragrunt plan for {}...", provider);
    let plan_output = Command::new("terragrunt")
        .arg("plan")
        .arg("--all")
        .arg("-out")
        .arg("out.tfplan")
        .current_dir(&env_path)
        .env("TF_IN_AUTOMATION", "true")
        .env("CHECKPOINT_DISABLE", "true")
        .output()
        .with_context(|| format!("Failed to run terragrunt plan for {}", provider))?;

    // Always generate fixture files, even if plan fails
    let cache_path = format!("{}/.terragrunt-cache", env_path);
    generate_modules_json(provider, &cache_path)?;
    
    // Always try to find and convert existing .tfplan files first
    // This handles cases where plan failed but we have tfplan files from previous runs
    if let Ok(()) = generate_plan_json(provider, &cache_path) {
        // Successfully found and converted .tfplan files
    } else if !plan_output.status.success() {
        let stderr = String::from_utf8_lossy(&plan_output.stderr);
        eprintln!("⚠️ Warning: terragrunt plan failed for {}: {}", provider, stderr);
        // Only create minimal plan file if we couldn't find any .tfplan files AND plan failed
        create_minimal_plan_json(provider)?;
    }

    println!("✅ Fixtures generated successfully for {} provider", provider);
    Ok(())
}

/// Generates modules.json file from module directory structure
/// 
/// This function dynamically discovers all modules by walking the filesystem
/// and creates a modules.json file that matches terragrunt's expected format.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// * `cache_path` - Path to terragrunt cache directory (currently unused)
/// 
/// # Returns
/// Result indicating success or failure of modules.json generation
/// 
/// # Module Discovery Methods
/// This function uses filesystem-based discovery which works well because:
/// - Fast and reliable (no external dependencies)
/// - Always reflects current state
/// - Provider-agnostic
/// 
/// Alternative approaches could include:
/// - `terragrunt graph dependencies` for dependency-aware discovery
/// - `terragrunt run-all plan --terragrunt-working-dir` for runtime discovery
/// - Parsing `.terragrunt` files for more sophisticated module relationships
fn generate_modules_json(provider: &str, _cache_path: &str) -> Result<()> {
    let modules_path = format!("simulator/{}/modules", provider);
    
    // Start with root module in a vector
    let mut all_modules = vec![r#"{"Key":"","Source":"","Dir":"."}"#.to_string()];
    
    // Dynamically discover all modules by walking the filesystem
    if Path::new(&modules_path).exists() {
        let mut discovered_modules = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&modules_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(module_name) = entry.file_name().to_str() {
                        // Skip hidden directories and common non-module directories
                        if !module_name.starts_with('.') && !module_name.starts_with('_') {
                            discovered_modules.push(format!(
                                r#"{{"Key":"{}","Source":"./modules/{}","Dir":"modules/{}"}}"#,
                                module_name, module_name, module_name
                            ));
                        }
                    }
                }
            }
        }
        
        // Sort modules for consistent output
        discovered_modules.sort();
        all_modules.extend(discovered_modules);
    }
    
    // Build proper JSON structure
    let modules_json = format!(r#"{{"Modules":[{}]}}"#, all_modules.join(","));

    // Write the dynamically generated modules.json
    let fixtures_dir = format!("tests/fixtures/{}", provider);
    fs::create_dir_all(&fixtures_dir)?;
    fs::write(format!("{}/modules.json", fixtures_dir), modules_json)?;
    
    println!("📁 Dynamically discovered and generated modules.json for {} provider", provider);
    
    Ok(())
}

/// Recursively searches for .tfplan files in a directory tree
/// 
/// This helper function recursively traverses directory structures to find
/// terraform plan files (.tfplan) at any depth within the terragrunt cache.
/// 
/// # Arguments
/// * `dir` - Directory to search recursively
/// * `plan_files` - Mutable vector to collect found plan file paths
fn find_tfplan_files_recursive(dir: &Path, plan_files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Recursively search subdirectories
                find_tfplan_files_recursive(&path, plan_files);
            } else if path.extension().map(|e| e == "tfplan").unwrap_or(false) {
                // Found a .tfplan file
                plan_files.push(path);
            }
        }
    }
}

/// Generates plan.json file from terraform plan files
/// 
/// Searches for .tfplan files in the terragrunt cache and converts them to JSON
/// format using terragrunt show. This internal function is used by the fixture
/// generation process.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// * `cache_path` - Path to terragrunt cache directory
/// 
/// # Returns
/// Result indicating success or failure of plan.json generation
fn generate_plan_json(provider: &str, cache_path: &str) -> Result<()> {
    // Recursively find all .tfplan files in the cache directory
    let mut plan_files = Vec::new();
    find_tfplan_files_recursive(Path::new(cache_path), &mut plan_files);
    
    // Try to convert the first plan file found
    for plan_file in plan_files {
        if let Some(plan_dir) = plan_file.parent() {
            if let Some(plan_filename) = plan_file.file_name().and_then(|n| n.to_str()) {
                // Use terragrunt show to convert plan to JSON
                let output = Command::new("terragrunt")
                    .arg("show")
                    .arg("-json")
                    .arg(plan_filename) // Use relative filename instead of full path
                    .current_dir(plan_dir)
                    .output();
                
                if let Ok(output) = output {
                    if output.status.success() {
                        let fixtures_dir = format!("tests/fixtures/{}", provider);
                        fs::create_dir_all(&fixtures_dir)?;
                        fs::write(format!("{}/out.json", fixtures_dir), output.stdout)?;
                        println!("✅ Generated out.json for {} provider from {}", provider, plan_file.display());
                        return Ok(());
                    } else {
                        eprintln!("⚠️ Warning: terragrunt show failed for {}: {}", 
                                 plan_file.display(), 
                                 String::from_utf8_lossy(&output.stderr));
                    }
                }
            }
        }
    }
    
    // If no plan file found or conversion failed, return an error
    // Let the caller decide whether to create minimal JSON
    anyhow::bail!("No .tfplan files found in {} or conversion failed", cache_path)
}

/// Validates terraform formatting for a provider's configuration
/// 
/// Runs `terraform fmt -check` to verify that all terraform files in the
/// provider's directory are properly formatted according to terraform standards.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// 
/// # Returns
/// Result indicating success if formatting is correct, error if not
/// 
/// # Errors
/// - Provider directory doesn't exist
/// - Terraform command execution failure
/// - Formatting violations found
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::validate_terraform_format;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_terraform_format("aws")?;
/// # Ok(())
/// # }
/// ```
pub fn validate_terraform_format(provider: &str) -> Result<()> {
    println!("📝 Checking Terraform formatting for {}...", provider);
    
    let simulator_path = format!("simulator/{}", provider);
    if !Path::new(&simulator_path).exists() {
        anyhow::bail!("Provider directory does not exist: {}", simulator_path);
    }

    let output = Command::new("terraform")
        .arg("fmt")
        .arg("-check")
        .arg("-recursive")
        .arg(&simulator_path)
        .output()
        .with_context(|| format!("Failed to run terraform fmt for {}", provider))?;

    if output.status.success() {
        println!("✅ Terraform formatting is correct for {}", provider);
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Terraform formatting check failed for {}\nStdout: {}\nStderr: {}",
            provider,
            stdout.trim(),
            stderr.trim()
        );
    }
}

/// Validates terraform configuration for a provider
/// 
/// Runs `terraform init` and `terraform validate` to verify that the terraform
/// configuration is syntactically correct and internally consistent.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// 
/// # Returns
/// Result indicating success if validation passes, error if not
/// 
/// # Errors
/// - Provider directory doesn't exist
/// - Terraform init failure
/// - Terraform validate failure
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::validate_terraform_config;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_terraform_config("gcp")?;
/// # Ok(())
/// # }
/// ```
pub fn validate_terraform_config(provider: &str) -> Result<()> {
    println!("✅ Running terraform validate for {}...", provider);
    
    let simulator_path = format!("simulator/{}", provider);
    if !Path::new(&simulator_path).exists() {
        anyhow::bail!("Provider directory does not exist: {}", simulator_path);
    }

    // First run terraform init -backend=false
    println!("🔧 Running terraform init for validation...");
    let init_output = Command::new("terraform")
        .arg("init")
        .arg("-backend=false")
        .current_dir(&simulator_path)
        .env("AWS_EC2_METADATA_DISABLED", "true")
        .env("TF_IN_AUTOMATION", "true")
        .env("CHECKPOINT_DISABLE", "true")
        .output()
        .with_context(|| format!("Failed to run terraform init for {}", provider))?;

    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        anyhow::bail!("Terraform init failed for {}: {}", provider, stderr.trim());
    }

    // Then run terraform validate
    let validate_output = Command::new("terraform")
        .arg("validate")
        .current_dir(&simulator_path)
        .env("AWS_EC2_METADATA_DISABLED", "true")
        .env("TF_IN_AUTOMATION", "true")
        .env("CHECKPOINT_DISABLE", "true")
        .output()
        .with_context(|| format!("Failed to run terraform validate for {}", provider))?;

    if validate_output.status.success() {
        println!("✅ Terraform validation passed for {}", provider);
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&validate_output.stdout);
        let stderr = String::from_utf8_lossy(&validate_output.stderr);
        anyhow::bail!(
            "Terraform validation failed for {}\nStdout: {}\nStderr: {}",
            provider,
            stdout.trim(),
            stderr.trim()
        );
    }
}

/// Formats terraform files for a provider or checks formatting
/// 
/// Runs `terraform fmt` to either format terraform files in place or check
/// if formatting is correct without making changes.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// * `check_only` - If true, only checks formatting; if false, fixes formatting
/// 
/// # Returns
/// Result indicating success or failure
/// 
/// # Errors
/// - Provider directory doesn't exist
/// - Terraform command execution failure
/// - Formatting violations found (in check mode)
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::format_terraform_files;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Fix formatting
/// format_terraform_files("aws", false)?;
/// 
/// // Check formatting only
/// format_terraform_files("aws", true)?;
/// # Ok(())
/// # }
/// ```
pub fn format_terraform_files(provider: &str, check_only: bool) -> Result<()> {
    let action = if check_only { "Checking" } else { "Fixing" };
    println!("🔧 {} Terraform formatting for {}...", action, provider);
    
    let simulator_path = format!("simulator/{}", provider);
    if !Path::new(&simulator_path).exists() {
        anyhow::bail!("Provider directory does not exist: {}", simulator_path);
    }

    let mut cmd = Command::new("terraform");
    cmd.arg("fmt");
    
    if check_only {
        cmd.arg("-check");
    }
    
    cmd.arg("-recursive").arg(&simulator_path);

    let output = cmd.output()
        .with_context(|| format!("Failed to run terraform fmt for {}", provider))?;

    if output.status.success() {
        if check_only {
            println!("✅ Terraform formatting is correct for {}", provider);
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("🔧 Formatted files for {}:\n{}", provider, stdout.trim());
            } else {
                println!("✅ No formatting changes needed for {}", provider);
            }
        }
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if check_only {
            anyhow::bail!(
                "Terraform formatting check failed for {}\nFiles need formatting:\n{}\n{}",
                provider,
                stdout.trim(),
                stderr.trim()
            );
        } else {
            anyhow::bail!(
                "Terraform format failed for {}\nStdout: {}\nStderr: {}",
                provider,
                stdout.trim(),
                stderr.trim()
            );
        }
    }
}

/// Initializes terragrunt modules for a provider and environment
/// 
/// Runs `terragrunt init --all` to initialize all modules in the specified
/// provider and environment. Supports safe mode for CI environments.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// * `env` - Environment name (typically "dev")
/// * `safe_mode` - If true, continues execution despite init failures
/// 
/// # Returns
/// Result indicating success or failure (always success in safe mode)
/// 
/// # Errors
/// - Environment path doesn't exist
/// - Terragrunt init failure (only in non-safe mode)
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::init_terragrunt;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// init_terragrunt("aws", "dev", false)?;
/// init_terragrunt("gcp", "dev", true)?; // Safe mode
/// # Ok(())
/// # }
/// ```
pub fn init_terragrunt(provider: &str, env: &str, safe_mode: bool) -> Result<()> {
    println!("🚀 Initializing terragrunt for {} (env: {})...", provider, env);
    
    let env_path = format!("envs/simulator/{}/{}", provider, env);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    // First clean the workspace
    clean_workspace(Some(provider), false)?;

    // Then run terragrunt init
    let output = Command::new("terragrunt")
        .arg("init")
        .arg("--all")
        .current_dir(&env_path)
        .env("TF_IN_AUTOMATION", "true")
        .env("CHECKPOINT_DISABLE", "true")
        .output()
        .with_context(|| format!("Failed to run terragrunt init for {}", provider))?;

    if output.status.success() {
        println!("✅ Terragrunt init succeeded for {}", provider);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = format!("Terragrunt init failed for {}: {}", provider, stderr.trim());
        
        if safe_mode {
            eprintln!("⚠️ Warning (safe mode): {}", error_msg);
            Ok(())
        } else {
            anyhow::bail!(error_msg);
        }
    }
}

/// Plans terraform changes for a provider and environment
/// 
/// Runs `terragrunt run-all plan` to generate execution plans for all modules
/// in the specified provider and environment. Supports environment variables
/// and safe mode for CI environments.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// * `env` - Environment name (typically "dev")
/// * `vars` - Optional environment variables in "KEY=value" format
/// * `safe_mode` - If true, continues execution despite plan failures
/// 
/// # Returns
/// Result indicating success or failure (always success in safe mode)
/// 
/// # Errors
/// - Environment path doesn't exist
/// - Terragrunt plan failure (only in non-safe mode)
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::plan_terragrunt;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// plan_terragrunt("aws", "dev", None, false)?;
/// plan_terragrunt("gcp", "dev", Some("PROJECT_ID=my-project"), true)?; // With vars and safe mode
/// # Ok(())
/// # }
/// ```
pub fn plan_terragrunt(provider: &str, env: &str, vars: Option<&str>, safe_mode: bool) -> Result<()> {
    println!("📋 Planning terragrunt for {} (env: {})...", provider, env);
    
    let env_path = format!("envs/simulator/{}/{}", provider, env);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    let mut cmd = Command::new("terragrunt");
    cmd.arg("plan")
        .arg("--all")
        .arg("-out")
        .arg("out.tfplan")
        .current_dir(&env_path)
        .env("AWS_EC2_METADATA_DISABLED", "true")
        .env("TF_IN_AUTOMATION", "true")
        .env("CHECKPOINT_DISABLE", "true");

    // Add environment variables if provided
    if let Some(vars_str) = vars {
        for var_pair in vars_str.split_whitespace() {
            if let Some((key, value)) = var_pair.split_once('=') {
                cmd.env(key, value);
            }
        }
    }

    let output = cmd.output()
        .with_context(|| format!("Failed to run terragrunt plan for {}", provider))?;

    // Extract plan summary from stdout and stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Look for plan summaries in both stdout and stderr
    if let Some(summary) = extract_plan_summary(&stdout).or_else(|| extract_plan_summary(&stderr)) {
        println!("📊 {}", summary);
    }

    if output.status.success() {
        println!("✅ Terragrunt plan succeeded for {}", provider);
        Ok(())
    } else {
        let error_msg = format!("Terragrunt plan failed for {}: {}", provider, stderr.trim());
        
        if safe_mode {
            eprintln!("⚠️ Warning (safe mode): {}", error_msg);
            Ok(())
        } else {
            anyhow::bail!(error_msg);
        }
    }
}

/// Applies terraform changes for a provider and environment
/// 
/// Runs `terragrunt run-all apply` to apply planned changes for all modules
/// in the specified provider and environment. This will create/modify/destroy
/// real cloud resources based on the terraform configuration.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// * `env` - Environment name (typically "dev")
/// * `auto_approve` - If true, skips confirmation prompt (default: false for safety)
/// * `safe_mode` - If true, continues execution despite apply failures
/// 
/// # Returns
/// Result indicating success or failure (always success in safe mode)
/// 
/// # Errors
/// - Environment path doesn't exist
/// - Terragrunt apply failure (only in non-safe mode)
/// 
/// # Warning
/// This function modifies real cloud resources. Use with caution!
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::apply_terragrunt;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// apply_terragrunt("aws", "dev", false, false)?; // Requires confirmation
/// apply_terragrunt("gcp", "dev", true, true)?; // Auto-approve with safe mode
/// # Ok(())
/// # }
/// ```
pub fn apply_terragrunt(provider: &str, env: &str, auto_approve: bool, safe_mode: bool) -> Result<()> {
    println!("🚀 Applying terragrunt for {} (env: {})...", provider, env);
    
    let env_path = format!("envs/simulator/{}/{}", provider, env);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    let mut cmd = Command::new("terragrunt");
    cmd.arg("apply")
        .arg("--all")
        .current_dir(&env_path)
        .env("AWS_EC2_METADATA_DISABLED", "true")
        .env("TF_IN_AUTOMATION", "true")
        .env("CHECKPOINT_DISABLE", "true");

    if auto_approve {
        cmd.arg("-auto-approve");
    }

    let output = cmd.output()
        .with_context(|| format!("Failed to run terragrunt apply for {}", provider))?;

    // Extract plan summary from stdout and stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Look for plan summaries in both stdout and stderr
    if let Some(summary) = extract_plan_summary(&stdout).or_else(|| extract_plan_summary(&stderr)) {
        println!("📊 {}", summary);
    }

    if output.status.success() {
        println!("✅ Terragrunt apply succeeded for {}", provider);
        Ok(())
    } else {
        let error_msg = format!("Terragrunt apply failed for {}: {}", provider, stderr.trim());
        
        if safe_mode {
            eprintln!("⚠️ Warning (safe mode): {}", error_msg);
            Ok(())
        } else {
            anyhow::bail!(error_msg);
        }
    }
}

/// Destroys terraform resources for a provider and environment
/// 
/// Runs `terragrunt run-all destroy` to destroy all resources managed by
/// terraform for the specified provider and environment. This will permanently
/// delete cloud resources.
/// 
/// # Arguments
/// * `provider` - Provider name (aws, gcp, azure)
/// * `env` - Environment name (typically "dev")
/// * `auto_approve` - If true, skips confirmation prompt (default: false for safety)
/// * `safe_mode` - If true, continues execution despite destroy failures
/// 
/// # Returns
/// Result indicating success or failure (always success in safe mode)
/// 
/// # Errors
/// - Environment path doesn't exist
/// - Terragrunt destroy failure (only in non-safe mode)
/// 
/// # Warning
/// This function permanently deletes real cloud resources. Use with extreme caution!
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::utils::destroy_terragrunt;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// destroy_terragrunt("aws", "dev", false, false)?; // Requires confirmation
/// destroy_terragrunt("gcp", "dev", true, true)?; // Auto-approve with safe mode
/// # Ok(())
/// # }
/// ```
pub fn destroy_terragrunt(provider: &str, env: &str, auto_approve: bool, safe_mode: bool) -> Result<()> {
    println!("💥 Destroying terragrunt for {} (env: {})...", provider, env);
    
    let env_path = format!("envs/simulator/{}/{}", provider, env);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    let mut cmd = Command::new("terragrunt");
    cmd.arg("destroy")
        .arg("--all")
        .current_dir(&env_path)
        .env("AWS_EC2_METADATA_DISABLED", "true")
        .env("TF_IN_AUTOMATION", "true")
        .env("CHECKPOINT_DISABLE", "true");

    if auto_approve {
        cmd.arg("-auto-approve");
    }

    let output = cmd.output()
        .with_context(|| format!("Failed to run terragrunt destroy for {}", provider))?;

    // Extract plan summary from stdout and stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Look for plan summaries in both stdout and stderr
    if let Some(summary) = extract_plan_summary(&stdout).or_else(|| extract_plan_summary(&stderr)) {
        println!("📊 {}", summary);
    }

    if output.status.success() {
        println!("✅ Terragrunt destroy succeeded for {}", provider);
        Ok(())
    } else {
        let error_msg = format!("Terragrunt destroy failed for {}: {}", provider, stderr.trim());
        
        if safe_mode {
            eprintln!("⚠️ Warning (safe mode): {}", error_msg);
            Ok(())
        } else {
            anyhow::bail!(error_msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_plan_summary() {
        // Test plan summary extraction
        let output_with_plan = "Some other output\nPlan: 5 to add, 3 to change, 1 to destroy.\nMore output";
        assert_eq!(
            extract_plan_summary(output_with_plan),
            Some("Plan: 5 to add, 3 to change, 1 to destroy.".to_string())
        );

        // Test no changes case
        let output_no_changes = "Other text\nNo changes. Your infrastructure matches the configuration.\nMore text";
        assert_eq!(
            extract_plan_summary(output_no_changes),
            Some("No changes. Your infrastructure matches the configuration.".to_string())
        );

        // Test apply complete case
        let output_apply = "Various output\nApply complete! Resources: 5 added, 3 changed, 1 destroyed.\nDone";
        assert_eq!(
            extract_plan_summary(output_apply),
            Some("Apply complete! Resources: 5 added, 3 changed, 1 destroyed.".to_string())
        );

        // Test no summary found
        let output_no_summary = "Just some regular output\nwithout any plan summary";
        assert_eq!(extract_plan_summary(output_no_summary), None);
    }
}
