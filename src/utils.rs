use crate::importer::{PlannedModule, Resource};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

pub use crate::schema::write_provider_schema;

#[derive(Error, Debug)]
pub enum TerragruntProcessError {
    #[error("Failed to run terragrunt: status={status}, stdout={stdout}, stderr={stderr}")]
    ProcessError {
        status: i32,
        stdout: String,
        stderr: String,
    },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

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

pub fn run_terragrunt_init(working_directory: &str) -> Result<(), TerragruntProcessError> {
    println!("🔧 Running `terragrunt init` in {}", working_directory);

    let output = Command::new("terragrunt")
        .arg("init")
        .current_dir(working_directory)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(TerragruntProcessError::ProcessError {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

pub fn clean_workspace() {
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
    remove_files_by_name(root, "out.tfplan");
    remove_files_by_name(Path::new("envs/simulator/dev"), "plan.json");
    remove_files_by_name(Path::new("envs/simulator/dev"), ".terragrunt-provider-schema.json");
}

pub fn perform_just_gen() {
    // Clean
    clean_workspace();

    // Init
    let _ = Command::new("terragrunt")
        .arg("init")
        .arg("--all")
        .current_dir("envs/simulator/dev")
        .output();

    // Plan
    let _ = Command::new("terragrunt")
        .arg("run-all")
        .arg("plan")
        .arg("-out")
        .arg("out.tfplan")
        .current_dir("envs/simulator/dev")
        .output();

    // Plans-to-JSON
    if let Ok(entries) = fs::read_dir("envs/simulator/dev/.terragrunt-cache") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.extension().map(|e| e == "tfplan").unwrap_or(false) {
                            let output = Command::new("terraform")
                                .arg("-chdir")
                                .arg(path.to_str().unwrap())
                                .arg("show")
                                .arg("-json")
                                .arg(sub_path.file_name().unwrap().to_str().unwrap())
                                .output();
                            if let Ok(output) = output {
                                let json_path = format!("test/tmp/{}.json", sub_path.file_stem().unwrap().to_str().unwrap());
                                let _ = fs::write(json_path, output.stdout);
                            }
                        }
                    }
                }
            }
        }
    }

    // Copy-Plan-JSON
    if let Ok(entries) = fs::read_dir("envs/simulator/dev/.terragrunt-cache") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.extension().map(|e| e == "json").unwrap_or(false) {
                            let dest_path = format!("tests/fixtures/{}", sub_path.file_name().unwrap().to_str().unwrap());
                            let _ = fs::copy(&sub_path, dest_path);
                        }
                    }
                }
            }
        }
    }
}
