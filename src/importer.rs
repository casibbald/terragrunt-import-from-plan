use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct ModulesFile {
    #[serde(rename = "Modules")]
    pub(crate) modules: Vec<ModuleMeta>,
}

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
pub struct RootModule {
    pub child_modules: Vec<ChildModule>,
}

#[derive(Debug, Deserialize)]
pub struct ChildModule {
    pub address: String,
    pub resources: Vec<Resource>,
}

#[derive(Debug, Deserialize)]
pub struct Plan {
    pub format_version: String,
    pub terraform_version: String,
    pub variables: Option<HashMap<String, Variable>>,
    pub planned_values: Option<PlannedValues>,
    pub resource_changes: Option<Vec<ResourceChange>>,
    pub configuration: Option<Configuration>,
    pub provider_schemas: Option<ProviderSchemas>,
}

#[derive(Debug, Deserialize)]
pub struct Variable {
    pub value: serde_json::Value,
    pub sensitive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PlannedValues {
    pub root_module: PlannedModule,
}

// For out.json (planned_values)
#[derive(Debug, Deserialize)]
pub struct PlannedModule {
    pub resources: Option<Vec<Resource>>,
    pub child_modules: Option<Vec<PlannedModule>>,
    pub address: Option<String>,
}

// For modules.json
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
pub struct Resource {
    pub address: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub provider_name: Option<String>,
    pub schema_version: Option<u32>,
    pub values: Option<serde_json::Value>,
    pub sensitive_values: Option<serde_json::Value>,
    pub depends_on: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceChange {
    pub address: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub provider_name: String,
    pub change: Change,
}

#[derive(Debug, Deserialize)]
pub struct Change {
    pub actions: Vec<String>,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
    pub after_unknown: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub provider_config: Option<HashMap<String, ProviderConfig>>,
    pub root_module: Option<ConfigModule>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub full_name: String,
    pub expressions: Option<HashMap<String, Expression>>,
}

#[derive(Debug, Deserialize)]
pub struct Expression {
    pub constant_value: Option<serde_json::Value>,
    pub references: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigModule {
    pub resources: Option<Vec<ConfigResource>>,
    pub child_modules: Option<Vec<ConfigModule>>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigResource {
    pub address: String,
    pub mode: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub provider_config_key: Option<String>,
    pub expressions: Option<HashMap<String, Expression>>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderSchemas {
    pub provider_schemas: HashMap<String, ProviderSchema>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderSchema {
    pub provider: Option<Schema>,
    pub resource_schemas: Option<HashMap<String, Schema>>,
    pub data_source_schemas: Option<HashMap<String, Schema>>,
}

#[derive(Debug, Deserialize)]
pub struct Schema {
    pub version: u32,
    pub block: Block,
}

#[derive(Debug, Deserialize)]
pub struct Block {
    pub attributes: Option<HashMap<String, Attribute>>,
    pub block_types: Option<HashMap<String, BlockType>>,
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    #[serde(rename = "type")]
    pub type_: serde_json::Value,
    pub optional: Option<bool>,
    pub required: Option<bool>,
    pub computed: Option<bool>,
    pub sensitive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BlockType {
    pub nesting_mode: String,
    pub block: Block,
    pub min_items: Option<u32>,
    pub max_items: Option<u32>,
}

pub fn validate_module_dirs<P: AsRef<Path>>(
    modules: &[crate::importer::ModuleMeta],
    base_path: P,
) -> Vec<String> {
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
pub fn map_resources_to_modules<'a>(
    modules: &'a [ModuleMeta],
    plan: &'a PlanFile,
) -> HashMap<String, &'a ModuleMeta> {
    let mut mapping = HashMap::new();

    if let Some(planned_values) = &plan.planned_values {
        if let Some(child_modules) = &planned_values.root_module.child_modules {
            for child in child_modules {
                if let Some(resources) = &child.resources {
                    if let Some(address) = &child.address {
                        let prefix = address.strip_prefix("module.").unwrap_or("");
                        if let Some(module) = modules.iter().find(|m| m.key == prefix) {
                            for resource in resources {
                                mapping.insert(resource.address.clone(), module);
                            }
                        }
                    }
                }
            }
        }
    }

    mapping
}

pub fn generate_import_commands(resource_map: &HashMap<String, &ModuleMeta>) -> Vec<String> {
    resource_map
        .iter()
        .map(|(resource_address, module_meta)| {
            format!(
                "terraform import -config-dir={} {} <RESOURCE_ID>",
                module_meta.dir, resource_address
            )
        })
        .collect()
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
        let modules_data =
            fs::read_to_string("tests/fixtures/modules.json").expect("Unable to read modules file");
        let plan_data =
            fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");

        let modules_file: ModulesFile =
            serde_json::from_str(&modules_data).expect("Invalid modules JSON");
        let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

        let mapping = map_resources_to_modules(&modules_file.modules, &plan);

        assert!(!mapping.is_empty(), "No resource-module mappings found");
    }

    #[test]
    fn test_generate_import_commands() {
        let modules_data =
            fs::read_to_string("tests/fixtures/modules.json").expect("Unable to read modules file");
        let plan_data =
            fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");

        let modules_file: ModulesFile =
            serde_json::from_str(&modules_data).expect("Invalid modules JSON");
        let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

        let mapping = map_resources_to_modules(&modules_file.modules, &plan);
        let commands = generate_import_commands(&mapping);

        assert!(!commands.is_empty(), "No import commands generated");
        for cmd in commands {
            assert!(
                cmd.starts_with("terraform import"),
                "Command does not start with terraform import: {}",
                cmd
            );
        }
    }

    #[test]
    fn test_infer_resource_id() {
        let plan_data =
            fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");
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
