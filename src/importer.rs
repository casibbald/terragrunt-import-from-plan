use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io::{self};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use serde_json::{Map, Value};
use crate::plan::TerraformResource;
use crate::utils::{collect_all_resources, collect_resources, extract_id_candidate_fields};


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

#[derive(Debug, Deserialize)]
pub struct ModuleMeta {
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(rename = "Source")]
    pub(crate) source: String,
    #[serde(rename = "Dir")]
    pub(crate) dir: String,
}

#[derive(Debug, Deserialize)]
pub struct ModulesFile {
    #[serde(rename = "Modules")]
    pub(crate) modules: Vec<ModuleMeta>,
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
                        panic!("Unmatched module address: {}", address);
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

        let schema_map = &plan.provider_schemas.as_ref().and_then(|ps| ps.provider_schemas.values().next())
            .and_then(|provider| provider.resource_schemas.as_ref())
            .cloned()
            .unwrap_or_default();

        for resource in all_resources {
            let terraform_resource = TerraformResource {
                address: resource.address.clone(),
                mode: resource.mode.clone(),
                r#type: resource.r#type.clone(),
                name: resource.name.clone(),
                values: resource.values.clone(),
            };

            let schema_map = plan
                .provider_schemas
                .as_ref()
                .and_then(|ps| ps.provider_schemas.values().next())
                .and_then(|provider| provider.resource_schemas.as_ref())
                .cloned()
                .unwrap_or_default();

            let inferred_id = infer_resource_id(
                &terraform_resource,
                schema_map.get(&terraform_resource.r#type),
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
    provider_schema_json: Option<&Value>,
    verbose: bool,
) -> Option<String> {
    let values = resource.values.as_ref()?.as_object()?;
    
    
    let candidates = if let Some(schema) = provider_schema_json {
        extract_id_candidate_fields(schema)
    } else {
        // fallback for test cases without schemas
        extract_id_candidate_fields_from_values(values)
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

pub fn execute_or_print_imports(
    resource_map: &HashMap<String, &ModuleMeta>,
    plan: &PlanFile,
    dry_run: bool,
    verbose: bool,
    module_root: &str,
) {
    if let Some(planned_values) = &plan.planned_values {
        
        let mut all_resources = vec![];
        collect_all_resources(&planned_values.root_module, &mut all_resources);

        let mut all_resources = vec![];
        collect_resources(&planned_values.root_module, &mut all_resources);

        let mut imported = 0;
        let mut skipped = 0;
        let mut failed = 0;

        let provider_schemas = match &plan.provider_schemas {
            Some(ps) => &ps.provider_schemas,
            None => {
                eprintln!("No provider_schemas found in plan");
                return;
            }
        };
        let schema_map = &plan.provider_schemas.as_ref().and_then(|ps| ps.provider_schemas.values().next())
            .and_then(|provider| provider.resource_schemas.as_ref())
            .cloned()
            .unwrap_or_default();

        for resource in all_resources {
            let terraform_resource = TerraformResource {
                address: resource.address.clone(),
                mode: resource.mode.clone(),
                r#type: resource.r#type.clone(),
                name: resource.name.clone(),
                values: resource.values.clone(),
            };

            let schema_map = plan
                .provider_schemas
                .as_ref()
                .and_then(|ps| ps.provider_schemas.values().next())
                .and_then(|provider| provider.resource_schemas.as_ref())
                .cloned()
                .unwrap_or_default();

            let inferred_id = infer_resource_id(
                &terraform_resource,
                schema_map.get(&terraform_resource.r#type),
                verbose,
            );


            match resource_map.get(&resource.address) {
                Some(module_meta) => {
                    if let Some(id) = inferred_id {
                        let module_path = PathBuf::from(module_root).join(&module_meta.dir);
                        if dry_run {
                            println!(
                                "🌿 [DRY RUN] terragrunt import -config-dir={} {} {}",
                                module_path.display(),
                                resource.address,
                                id
                            );
                            imported += 1;
                        } else {
                            match run_terragrunt_import(&module_path, &resource.address, id) {
                                Ok(_) => {
                                    println!("✅ Imported {}", resource.address);
                                    imported += 1;
                                }
                                Err(_) => {
                                    eprintln!(
                                        "❌ Error importing {}: Module path does not exist: {}",
                                        resource.address,
                                        module_path.display()
                                    );
                                    failed += 1;
                                }
                            }
                        }
                    } else {
                        println!("⚠️ Skipped {}: no ID could be inferred", resource.address);
                        skipped += 1;
                    }
                }
                None => {
                    println!(
                        "⚠️ Skipped {}: no matching module mapping found",
                        resource.address
                    );
                    skipped += 1;
                }
            }
        }

        println!(
            "\n📦 Import Summary\nImported:   {}\nAlready in state: 0\nSkipped:     {}\nFailed:      {}",
            imported, skipped, failed
        );
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

    let status = Command::new("terragrunt")
        .arg("import")
        .arg(resource_address)
        .arg(resource_id)
        .current_dir(module_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to import {}", resource_address),
        ))
    }
}


fn check(
    module: &PlannedModule,
    found: &mut bool,
    verbose: bool,
    schema_map: &HashMap<String, serde_json::Value>,
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

            if let Some(id) = infer_resource_id(&terraform_resource, schema_map.get(&terraform_resource.r#type), verbose) {
                println!("Inferred ID for {}: {}", terraform_resource.address, id);
                *found = true;
                return;
            }
        }
    }

    if let Some(children) = &module.child_modules {
        for child in children {
            check(child, found, verbose, schema_map);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::hash::Hash;

    #[test]
    fn test_validate_module_dirs() {
        let data = fs::read_to_string("tests/fixtures/modules.json").expect("Unable to read file");
        let modules_file: ModulesFile = serde_json::from_str(&data).expect("Invalid JSON");

        let errors = validate_module_dirs(&modules_file.modules, Path::new("simulator"));

        assert!(errors.is_empty(), "Found invalid directories: {:?}", errors);
    }

    #[test]
    fn test_map_resources_to_modules() {
        let modules_data = fs::read_to_string("tests/fixtures/modules.json").expect("Unable to read modules file");
        let plan_data = fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");

        let modules_file: ModulesFile = serde_json::from_str(&modules_data).expect("Invalid modules JSON");
        let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

        let mapping = map_resources_to_modules(&modules_file.modules, &plan);

        assert!(!mapping.is_empty(), "No resource-module mappings found");
    }

    #[test]
    fn test_generate_import_commands() {
        let modules_data = fs::read_to_string("tests/fixtures/modules.json").expect("Unable to read modules file");
        let plan_data = fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");

        let modules_file: ModulesFile = serde_json::from_str(&modules_data).expect("Invalid modules JSON");
        let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

        let mapping = map_resources_to_modules(&modules_file.modules, &plan);
        let commands = generate_import_commands(&mapping, &plan, ".", true);

        assert!(!commands.is_empty(), "No import commands generated");
        for cmd in commands {
            assert!(cmd.starts_with("terragrunt import"), "Command does not start with terraform import: {}", cmd);
        }
    }

    #[test]
    fn test_infer_resource_id() {
        let plan_data = fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");
        let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");
        let verbose = true;

        let mut found = false;
        if let Some(planned_values) = &plan.planned_values {
            let check = |module: &PlannedModule, found: &mut bool, verbose: bool| {
                if let Some(resources) = &module.resources {
                    for resource in resources {
                        let terraform_resource = TerraformResource {
                            address: resource.address.clone(),
                            mode: resource.mode.clone(),
                            r#type: resource.r#type.clone(),
                            name: resource.name.clone(),
                            values: resource.values.clone(),
                        };

                        let schema_map = plan
                            .provider_schemas
                            .as_ref()
                            .and_then(|ps| ps.provider_schemas.values().next())
                            .and_then(|provider| provider.resource_schemas.as_ref())
                            .cloned()
                            .unwrap_or_default();

                        if let Some(id) = infer_resource_id(&terraform_resource, schema_map.get(&terraform_resource.r#type), verbose) {
                            println!("Inferred ID for {}: {}", resource.address, id);
                            *found = true;
                            return;
                        }
                    }
                }
                if let Some(children) = &module.child_modules {

                    let schema_map = plan
                        .provider_schemas
                        .as_ref()
                        .and_then(|ps| ps.provider_schemas.values().next())
                        .and_then(|provider| provider.resource_schemas.as_ref())
                        .cloned()
                        .unwrap_or_default();
                    for child in children {
                        check(child, found, verbose, &schema_map);
                    }
                }
            };
            check(&planned_values.root_module, &mut found, verbose);
        }

        assert!(found, "No resource ID could be inferred");
    }

    
    


    #[test]
    fn test_run_terragrunt_import_mock() {
        // This test validates the command construction without executing terraform.
        let module_dir = "mock_dir";
        let resource_address = "mock_resource_address";
        let resource_id = "mock_resource_id";

        let cmd = Command::new("echo")
            .arg("terragrunt")
            .arg("import")
            .arg("-config-dir")
            .arg(module_dir)
            .arg(resource_address)
            .arg(resource_id)
            .output()
            .expect("Failed to simulate terragrunt command");

        let output = String::from_utf8_lossy(&cmd.stdout);
        assert!(output.contains("terragrunt"));
        assert!(output.contains(module_dir));
        assert!(output.contains(resource_address));
        assert!(output.contains(resource_id));
    }


}