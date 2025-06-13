use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{self};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use serde_json::{Map, Value};
use crate::plan::TerraformResource;
use crate::reporting::{ImportStats, ImportOperation, print_import_progress, print_import_summary};
use crate::utils::{collect_resources, extract_id_candidate_fields};


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
        collect_resources(&planned_values.root_module, &mut all_resources);

        let mut stats = ImportStats::new();

        for resource in all_resources {
            if verbose {
                print_import_progress(&resource.address, ImportOperation::Checking);
            }

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
                        
                        if verbose {
                            print_import_progress(&resource.address, ImportOperation::Importing { id: id.clone() });
                        }
                        
                        if dry_run {
                            let command = format!(
                                "terragrunt import -config-dir={} {} {}",
                                module_path.display(),
                                resource.address,
                                id
                            );
                            print_import_progress(&resource.address, ImportOperation::DryRun { command });
                            stats.increment_imported(resource.address.clone());
                        } else {
                            match run_terragrunt_import(&module_path, &resource.address, id) {
                                Ok(_) => {
                                    print_import_progress(&resource.address, ImportOperation::Success);
                                    stats.increment_imported(resource.address.clone());
                                }
                                Err(e) => {
                                    let error_msg = format!("Import failed: {}", e);
                                    print_import_progress(&resource.address, ImportOperation::Failed { error: error_msg });
                                    stats.increment_failed();
                                }
                            }
                        }
                    } else {
                        print_import_progress(&resource.address, ImportOperation::Skipped { 
                            reason: "no ID could be inferred".to_string() 
                        });
                        stats.increment_skipped();
                    }
                }
                None => {
                    print_import_progress(&resource.address, ImportOperation::Skipped { 
                        reason: "no matching module mapping found".to_string() 
                    });
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
    schema_map: &HashMap<String, Value>,
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

