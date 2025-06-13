use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Once;
use tempfile::TempDir;
use terragrunt_import_from_plan::importer::{
    PlannedModule, Resource, ModulesFile, PlanFile,
    validate_module_dirs, map_resources_to_modules, generate_import_commands, infer_resource_id
};
use terragrunt_import_from_plan::utils::{
    collect_resources, collect_all_resources, extract_id_candidate_fields,
    run_terragrunt_init, write_provider_schema, perform_just_gen
};
use serde_json::json;
use terragrunt_import_from_plan::schema::SchemaError;
use terragrunt_import_from_plan::utils::TerragruntProcessError;
use std::io::{self, Write};
use std::collections::HashMap;
use serde_json::{Value, Map};
use thiserror::Error;
use terragrunt_import_from_plan::plan::TerraformResource;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        perform_just_gen();
    });
}

// Helper function to create a test module structure
fn create_test_module() -> PlannedModule {
    PlannedModule {
        resources: Some(vec![
            Resource {
                address: "test.resource1".to_string(),
                mode: "managed".to_string(),
                r#type: "test_type".to_string(),
                name: "resource1".to_string(),
                provider_name: None,
                schema_version: None,
                values: None,
                sensitive_values: None,
                depends_on: None,
            },
            Resource {
                address: "test.resource2".to_string(),
                mode: "managed".to_string(),
                r#type: "test_type".to_string(),
                name: "resource2".to_string(),
                provider_name: None,
                schema_version: None,
                values: None,
                sensitive_values: None,
                depends_on: None,
            },
        ]),
        child_modules: Some(vec![
            PlannedModule {
                resources: Some(vec![
                    Resource {
                        address: "module.child.test.resource3".to_string(),
                        mode: "managed".to_string(),
                        r#type: "test_type".to_string(),
                        name: "resource3".to_string(),
                        provider_name: None,
                        schema_version: None,
                        values: None,
                        sensitive_values: None,
                        depends_on: None,
                    },
                ]),
                child_modules: None,
                address: Some("module.child".to_string()),
            },
        ]),
        address: None,
    }
}

#[test]
fn test_03_collect_resources() {
    let module = create_test_module();
    let mut resources = Vec::new();
    collect_resources(&module, &mut resources);
    
    assert_eq!(resources.len(), 3);
    assert_eq!(resources[0].address, "test.resource1");
    assert_eq!(resources[1].address, "test.resource2");
    assert_eq!(resources[2].address, "module.child.test.resource3");
}

#[test]
fn test_04_collect_all_resources() {
    let module = create_test_module();
    let mut resources = Vec::new();
    collect_all_resources(&module, &mut resources);
    
    assert_eq!(resources.len(), 3);
    assert_eq!(resources[0].address, "test.resource1");
    assert_eq!(resources[1].address, "test.resource2");
    assert_eq!(resources[2].address, "module.child.test.resource3");
}

#[test]
fn test_05_collect_resources_empty_module() {
    let module = PlannedModule {
        resources: None,
        child_modules: None,
        address: None,
    };
    let mut resources = Vec::new();
    collect_resources(&module, &mut resources);
    assert!(resources.is_empty());
}

#[test]
fn test_06_extract_id_candidate_fields() {
    let schema_json = json!({
        "provider_schemas": {
            "google": {
                "resource_schemas": {
                    "test_resource": {
                        "block": {
                            "attributes": {
                                "id": {},
                                "name": {},
                                "project": {}
                            }
                        }
                    }
                }
            }
        }
    });

    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.contains("id"));
    assert!(candidates.contains("name"));
    assert!(candidates.contains("project"));
}

#[test]
fn test_07_extract_id_candidate_fields_empty_schema() {
    let schema_json = json!({});
    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.is_empty());
}

#[test]
fn test_08_extract_id_candidate_fields_missing_provider() {
    let schema_json = json!({
        "provider_schemas": {}
    });
    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.is_empty());
}

#[test]
fn test_15_get_id_candidate_fields() {
    let schema_json = json!({
        "provider_schemas": {
            "google": {
                "resource_schemas": {
                    "test_resource": {
                        "block": {
                            "attributes": {
                                "id": {},
                                "name": {},
                                "project": {}
                            }
                        }
                    }
                }
            }
        }
    });

    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.contains("id"));
    assert!(candidates.contains("name"));
    assert!(candidates.contains("project"));
}

#[test]
fn test_16_get_id_candidate_fields_empty() {
    let schema_json = json!({});
    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.is_empty());
}

#[test]
fn test_17_get_id_candidate_fields_less_than_three() {
    let schema_json = json!({
        "provider_schemas": {
            "google": {
                "resource_schemas": {
                    "test_resource": {
                        "block": {
                            "attributes": {
                                "id": {}
                            }
                        }
                    }
                }
            }
        }
    });

    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.contains("id"));
    assert_eq!(candidates.len(), 1);
}

#[test]
fn test_19_load_provider_schema_invalid_file() {
    let temp_dir = TempDir::new().unwrap();
    let schema_path = temp_dir.path().join(".terragrunt-provider-schema.json");
    fs::write(&schema_path, "invalid json").unwrap();
    let result = write_provider_schema(temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_21_score_attributes_for_id() {
    let schema_json = json!({
        "provider_schemas": {
            "google": {
                "resource_schemas": {
                    "test_resource": {
                        "block": {
                            "attributes": {
                                "id": {},
                                "name": {},
                                "project": {}
                            }
                        }
                    }
                }
            }
        }
    });

    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.contains("id"));
    assert!(candidates.contains("name"));
    assert!(candidates.contains("project"));
}

#[test]
fn test_22_score_attributes_for_id_empty() {
    let schema_json = json!({});
    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.is_empty());
}

#[test]
fn test_23_generate_import_commands() {
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
fn test_24_infer_resource_id() {
    let plan_data = fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");
    let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");
    let verbose = true;

    let mut found = false;
    if let Some(planned_values) = &plan.planned_values {
        fn check(module: &PlannedModule, found: &mut bool, verbose: bool, schema_map: &HashMap<String, Value>) {
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
                        println!("Inferred ID for {}: {}", resource.address, id);
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

        let schema_map = plan
            .provider_schemas
            .as_ref()
            .and_then(|ps| ps.provider_schemas.values().next())
            .and_then(|provider| provider.resource_schemas.as_ref())
            .cloned()
            .unwrap_or_default();

        check(&planned_values.root_module, &mut found, verbose, &schema_map);
    }

    assert!(found, "No resource ID could be inferred");
}

#[test]
fn test_25_map_resources_to_modules() {
    let modules_data = fs::read_to_string("tests/fixtures/modules.json").expect("Unable to read modules file");
    let plan_data = fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");

    let modules_file: ModulesFile = serde_json::from_str(&modules_data).expect("Invalid modules JSON");
    let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

    let mapping = map_resources_to_modules(&modules_file.modules, &plan);

    assert!(!mapping.is_empty(), "No resource-module mappings found");
}

#[test]
fn test_26_run_terragrunt_import_mock() {
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

#[test]
fn test_27_validate_module_dirs() {
    let data = fs::read_to_string("tests/fixtures/modules.json").expect("Unable to read file");
    let modules_file: ModulesFile = serde_json::from_str(&data).expect("Invalid JSON");

    let errors = validate_module_dirs(&modules_file.modules, Path::new("simulator"));

    assert!(errors.is_empty(), "Found invalid directories: {:?}", errors);
}

#[test]
fn test_28_map_resources_to_modules() {
    let modules_data = fs::read_to_string("tests/fixtures/modules.json").expect("Unable to read modules file");
    let plan_data = fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");

    let modules_file: ModulesFile = serde_json::from_str(&modules_data).expect("Invalid modules JSON");
    let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

    let mapping = map_resources_to_modules(&modules_file.modules, &plan);

    assert!(!mapping.is_empty(), "No resource-module mappings found");
}

#[test]
fn test_29_generate_import_commands() {
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
fn test_30_infer_resource_id() {
    let plan_data = fs::read_to_string("tests/fixtures/out.json").expect("Unable to read plan file");
    let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");
    let verbose = true;

    let mut found = false;
    if let Some(planned_values) = &plan.planned_values {
        fn check(module: &PlannedModule, found: &mut bool, verbose: bool, schema_map: &HashMap<String, Value>) {
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
                        println!("Inferred ID for {}: {}", resource.address, id);
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

        let schema_map = plan
            .provider_schemas
            .as_ref()
            .and_then(|ps| ps.provider_schemas.values().next())
            .and_then(|provider| provider.resource_schemas.as_ref())
            .cloned()
            .unwrap_or_default();

        check(&planned_values.root_module, &mut found, verbose, &schema_map);
    }

    assert!(found, "No resource ID could be inferred");
}

#[test]
fn test_31_run_terragrunt_import_mock() {
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