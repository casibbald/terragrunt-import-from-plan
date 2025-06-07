#![feature(let_chains)]

mod errors;
mod importer;
mod plan;
mod utils;

use crate::importer::{map_resources_to_modules, generate_import_commands, ModulesFile, PlanFile, infer_resource_id, Resource, PlannedModule, run_terragrunt_import, execute_or_print_imports};
use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::id;
use crate::plan::TerraformResource;
use crate::utils::collect_resources;

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
}


fn load_modules<P: AsRef<Path>>(path: P) -> Result<ModulesFile, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let modules: ModulesFile = serde_json::from_str(&content)?;
    Ok(modules)
}

fn load_plan<P: AsRef<Path>>(path: P) -> Result<PlanFile, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let plan: PlanFile = serde_json::from_str(&content)?;
    Ok(plan)
}

fn main() {
    let args = Args::parse();

    let mut modules_file = load_modules(&args.modules).expect("Failed to load modules");
    let plan_file = load_plan(&args.plan).expect("Failed to load plan");

    let module_root = args.module_root.clone().unwrap_or_else(|| ".".to_string());
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
}
