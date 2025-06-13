#![feature(let_chains)]

mod app;
mod errors;
mod importer;
mod plan;
mod reporting;
mod schema;
mod utils;

use crate::app::load_input_files;
use crate::importer::{execute_or_print_imports, generate_import_commands, infer_resource_id, map_resources_to_modules, run_terragrunt_import};
use crate::plan::TerraformResource;
use crate::utils::{collect_resources, run_terragrunt_init, write_provider_schema};
use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;


#[derive(Parser, Debug)]
#[command(name = "terragrunt_import_from_plan")]
#[command(about = "Generates terragrunt import commands from a tf.plan JSON", long_about = None)]
struct Args {
    #[arg(long)]
    plan: String,

    #[arg(long)]
    modules: String,

    #[arg(long)]
    module_root: Option<String>,

    #[arg(long, default_value_t = false)]
    dry_run: bool,

    #[arg(long, default_value_t = false)]
    verbose: bool,

    #[arg(long,)]
    working_directory: Option<String>,
}




fn main() -> Result<()> {
    let args = Args::parse();

    let (modules_file, plan_file) = load_input_files(&args.modules, &args.plan)
        .context("Failed to load input files")?;
    let module_root = args.module_root.clone().unwrap_or_else(|| ".".to_string());
    // let module_root_path = Path::new(&module_root);

    // 🌐 Try to extract provider schema if possible
        if let Err(e) = run_terragrunt_init(args.working_directory.as_deref().unwrap_or(".")) {
            eprintln!("⚠️ Warning: terragrunt init failed: {:#}", e);
            // Continue execution despite the error
        }

        if let Err(e) = write_provider_schema(Path::new(args.working_directory.as_deref().unwrap_or("."))) {
            eprintln!("⚠️ Failed to generate provider schema: {}", e);
        }
    
    let mapping = map_resources_to_modules(&modules_file.modules, &plan_file);
    execute_or_print_imports(&mapping, &plan_file, args.dry_run, args.verbose, &module_root);

    if args.dry_run {
        let commands = generate_import_commands(&mapping, &plan_file, &module_root, args.dry_run);
        for cmd in commands {
            println!("{}", cmd);
        }
    } else {
        if let Some(planned_values) = &plan_file.planned_values {
            
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

                let schema_map = plan_file
                    .provider_schemas
                    .as_ref()
                    .and_then(|ps| ps.provider_schemas.values().next())
                    .and_then(|provider| provider.resource_schemas.as_ref())
                    .cloned()
                    .unwrap_or_default();

                let inferred_id = infer_resource_id(
                    &terraform_resource,
                    schema_map.get(&terraform_resource.r#type),
                    args.verbose,
                );


                if let Some(id) = inferred_id {
                    if let Some(module_meta) = mapping.get(&resource.address) {
                        let module_path = Path::new(&module_meta.dir);
                        match run_terragrunt_import(module_path, &resource.address, id) {
                            Ok(_) => println!("✅ Imported {}", resource.address),
                            Err(e) => eprintln!("❌ Error importing {}: {}", resource.address, e),
                        }
                    } else {
                        eprintln!("⚠️ Skipped {}: no matching module mapping", resource.address);
                    }
                } else {
                    eprintln!("⚠️ Skipped {}: no ID could be inferred", resource.address);
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::Once;
    use tempfile::TempDir;
    use terragrunt_import_from_plan::utils::{run_terragrunt_init, write_provider_schema};

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // Setup code here
        });
    }

    #[test]
    fn test_01_setup_and_init() {
        setup();
        let temp_dir = TempDir::new().unwrap();
        let result = run_terragrunt_init(temp_dir.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_02_init_invalid_dir() {
        setup();
        let result = run_terragrunt_init("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_09_write_provider_schema() {
        setup();
        let temp_dir = TempDir::new().unwrap();
        let result = write_provider_schema(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_10_write_provider_schema_invalid_dir() {
        setup();
        let result = write_provider_schema(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_11_write_provider_schema_terragrunt_not_found() {
        setup();
        let temp_dir = TempDir::new().unwrap();
        let result = Command::new("nonexistent_command")
            .arg("providers")
            .arg("schema")
            .arg("-json")
            .current_dir(temp_dir.path())
            .output();
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.raw_os_error().unwrap(), 2); // ENOENT
        } else {
            panic!("Expected error");
        }
    }

    #[test]
    fn test_12_error_context_formatting() {
        setup();
        let temp_dir = TempDir::new().unwrap();
        let result = run_terragrunt_init(temp_dir.path().to_str().unwrap());
        assert!(result.is_err());
        
        let error_string = format!("{:#}", result.unwrap_err());
        
        // Verify the error contains useful context
        // The error might be about terragrunt command not found rather than execution failure
        assert!(error_string.contains("terragrunt") && (
            error_string.contains("Failed to execute") || 
            error_string.contains("Terragrunt init failed") ||
            error_string.contains("command not found") ||
            error_string.contains("No such file")
        ));
        assert!(error_string.contains(temp_dir.path().to_str().unwrap()));
    }
}
