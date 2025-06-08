use crate::importer::{PlannedModule, Resource};
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn collect_resources<'a>(module: &'a PlannedModule, all: &mut Vec<&'a Resource>) {
    if let Some(res) = &module.resources {
        all.extend(res.iter());
    }
    if let Some(children) = &module.child_modules {
        for child in children {
            collect_resources(child, all);
        }
    }
}


pub fn collect_all_resources<'a>(module: &'a PlannedModule, resources: &mut Vec<&'a Resource>) {
    if let Some(rs) = &module.resources {
        resources.extend(rs.iter());
    }
    if let Some(children) = &module.child_modules {
        for child in children {
            collect_all_resources(child, resources);
        }
    }
}

pub fn extract_id_candidate_fields(schema_json: &Value) -> HashSet<String> {
    let mut candidates = HashSet::new();

    if let Some(resource_schemas) = schema_json
        .get("provider_schemas")
        .and_then(|ps| ps.get("google")) // assumes Google provider
        .and_then(|g| g.get("resource_schemas"))
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

    candidates
}

pub fn run_terragrunt_init(working_directory: &str) -> io::Result<()> {
    println!("🔧 Running `terragrunt init` in {}", working_directory);

    let status = Command::new("terragrunt")
        .arg("init")
        .current_dir(working_directory)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("`terragrunt init` failed in {}", working_directory)
        ))
    }
}



/// Runs `terragrunt providers schema -json` and writes the output to `.terragrunt-provider-schema.json`
/// inside the specified `module_root` directory.
pub fn write_provider_schema(working_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Extracting provider schema in {}", working_dir.display());

    let output = Command::new("terragrunt")
        .arg("providers")
        .arg("schema")
        .arg("-json")
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Terragrunt provider schema extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    let json = String::from_utf8(output.stdout)?;
    let file_path = working_dir.join(".terragrunt-provider-schema.json");
    let mut file = File::create(&file_path)?;
    file.write_all(json.as_bytes())?;

    println!("✅ Provider schema written to {}", file_path.display());
    Ok(())
}
