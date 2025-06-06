mod errors;
mod importer;
mod plan;
mod utils;

use crate::importer::ModulesFile;
use clap::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "tf.plan")]
    plan_path: String,
}

fn load_modules<P: AsRef<Path>>(path: P) -> Result<ModulesFile, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let modules: ModulesFile = serde_json::from_str(&content)?;
    Ok(modules)
}

fn main() {
    let path = "tests/fixtures/modules.json";
    match load_modules(path) {
        Ok(modules_file) => {
            for module in modules_file.modules {
                println!(
                    "Key: {}, Source: {}, Dir: {}",
                    module.key, module.source, module.dir
                );
            }
        }
        Err(e) => eprintln!("Failed to load modules: {}", e),
    }
}
