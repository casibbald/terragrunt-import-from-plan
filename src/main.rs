mod errors;
mod importer;
mod plan;
mod utils;

use crate::importer::{map_resources_to_modules, generate_import_commands, ModulesFile, PlanFile, infer_resource_id, Resource, PlannedModule, run_terragrunt_import};
use clap::Parser;
use std::fs;
use std::path::Path;

/// CLI arguments
#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "tests/fixtures/out.json")]
    plan: String,

    #[arg(short, long, default_value = "tests/fixtures/modules.json")]
    modules: String,

    #[arg(long, default_value_t = false)]
    dry_run: bool,
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

    let modules_file = load_modules(&args.modules).expect("Failed to load modules");
    let plan_file = load_plan(&args.plan).expect("Failed to load plan");

    let mapping = map_resources_to_modules(&modules_file.modules, &plan_file);

    if args.dry_run {
        let commands = generate_import_commands(&mapping);
        for cmd in commands {
            println!("{}", cmd);
        }
    } else {
        for (resource_address, module_meta) in &mapping {
            // Look up full Resource object from plan to infer ID
            let mut inferred_id = None;
            if let Some(planned_values) = &plan_file.planned_values {
                fn search<'a>(module: &'a PlannedModule, addr: &str) -> Option<&'a Resource> {
                    if let Some(resources) = &module.resources {
                        if let Some(found) = resources.iter().find(|r| r.address == addr) {
                            return Some(found);
                        }
                    }
                    if let Some(children) = &module.child_modules {
                        for child in children {
                            if let Some(found) = search(child, addr) {
                                return Some(found);
                            }
                        }
                    }
                    None
                }
                if let Some(resource) = search(&planned_values.root_module, resource_address) {
                    inferred_id = infer_resource_id(resource);
                }
            }

            if let Some(id) = inferred_id {
                match run_terragrunt_import(&module_meta.dir, resource_address, &id) {
                    Ok(_) => println!("✅ Imported {}", resource_address),
                    Err(e) => eprintln!("❌ Error importing {}: {}", resource_address, e),
                }
            } else {
                eprintln!("⚠️ Skipped {}: no ID could be inferred", resource_address);
            }
        }
    }
}

