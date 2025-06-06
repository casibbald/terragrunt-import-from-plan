use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::{self, Write};

#[derive(Debug, Deserialize)]
pub struct PlanFile {
    pub format_version: String,
    pub terraform_version: String,
    pub variables: Option<Variables>,
    pub planned_values: Option<PlannedValues>,
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

pub fn validate_module_dirs<P: AsRef<Path>>(modules: &[ModuleMeta], base_path: P) -> Vec<String> {
    modules
        .iter()
        .filter_map(|module| {
            let path = base_path.as_ref().join(&module.dir);
            if !path.is_dir() {
                Some(format!("Missing or invalid directory: {}", path.display()))
            } else {
                None
            }
        })
        .collect()
}

pub fn map_resources_to_modules<'a>(modules: &'a [ModuleMeta], plan: &'a PlanFile) -> HashMap<String, &'a ModuleMeta> {
    let mut mapping = HashMap::new();

    if let Some(planned_values) = &plan.planned_values {
        fn recurse_modules<'a>(modules: &'a [ModuleMeta], module: &'a PlannedModule, mapping: &mut HashMap<String, &'a ModuleMeta>) {
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

pub fn generate_import_commands(resource_map: &HashMap<String, &ModuleMeta>) -> Vec<String> {
    resource_map.iter().map(|(resource_address, module_meta)| {
        format!(
            "terraform import -config-dir={} {} <RESOURCE_ID>",
            module_meta.dir,
            resource_address
        )
    }).collect()
}

pub fn infer_resource_id(resource: &Resource) -> Option<String> {
    if let Some(values) = &resource.values {
        if let Some(id_val) = values.get("name") {
            return Some(id_val.to_string().replace('"', ""));
        }
        if let Some(id_val) = values.get("id") {
            return Some(id_val.to_string().replace('"', ""));
        }
    }
    None
}

pub fn run_terraform_import(module_dir: &str, resource_address: &str, resource_id: &str) -> io::Result<()> {
    let status = Command::new("terraform")
        .arg("import")
        .arg("-config-dir")
        .arg(module_dir)
        .arg(resource_address)
        .arg(resource_id)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, format!("Failed to import {}", resource_address)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
        let commands = generate_import_commands(&mapping);

        assert!(!commands.is_empty(), "No import commands generated");
        for cmd in commands {
            assert!(cmd.starts_with("terraform import"), "Command does not start with terraform import: {}", cmd);
        }
    }

    #[test]
    fn test_infer_resource_id() {
        let plan_data = fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");
        let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

        let mut found = false;
        if let Some(planned_values) = &plan.planned_values {
            fn check(module: &PlannedModule, found: &mut bool) {
                if let Some(resources) = &module.resources {
                    for resource in resources {
                        if let Some(id) = infer_resource_id(resource) {
                            println!("Inferred ID for {}: {}", resource.address, id);
                            *found = true;
                            return;
                        }
                    }
                }
                if let Some(children) = &module.child_modules {
                    for child in children {
                        check(child, found);
                    }
                }
            }
            check(&planned_values.root_module, &mut found);
        }

        assert!(found, "No resource ID could be inferred");
    }
}
