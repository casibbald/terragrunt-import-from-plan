mod errors;
mod importer;
mod plan;
mod utils;

use crate::importer::{map_resources_to_modules, generate_import_commands, ModulesFile, PlanFile};
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
    let commands = generate_import_commands(&mapping);

    if args.dry_run {
        for cmd in commands {
            println!("{}", cmd);
        }
    } else {
        println!("Non-dry-run execution not yet implemented");
    }
}
