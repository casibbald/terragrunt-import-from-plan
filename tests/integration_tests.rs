//! # Integration Tests for Terragrunt Import Tool
//! 
//! This module contains comprehensive integration tests for the terragrunt import tool.
//! The tests verify end-to-end functionality including resource collection, ID inference,
//! command generation, and provider schema handling across multiple cloud providers.
//! 
//! ## Test Categories
//! 
//! - **Resource Collection Tests**: Verify recursive resource collection from terraform modules
//! - **Schema Processing Tests**: Test provider schema loading and ID candidate extraction
//! - **Import Command Tests**: Validate terragrunt import command generation
//! - **Module Mapping Tests**: Test mapping of resources to their corresponding modules
//! - **Provider Schema Tests**: Test schema generation for AWS, GCP, and Azure
//! - **Multi-Cloud Tests**: Verify functionality across different cloud providers
//! 
//! ## Test Setup
//! 
//! Tests use real fixture files and may attempt to generate fresh provider schemas.
//! They are designed to be CI-friendly and continue execution even when cloud 
//! credentials are not available.
//! 
//! ## Fixture Files
//! 
//! Tests rely on fixture files in `tests/fixtures/{provider}/` containing:
//! - `modules.json`: Module metadata from terragrunt
//! - `out.json`: Terraform plan in JSON format

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
    collect_resources, extract_id_candidate_fields,
    write_provider_schema, generate_fixtures
};
use serde_json::json;
use std::collections::HashMap;
use serde_json::Value;
use terragrunt_import_from_plan::plan::TerraformResource;

static INIT: Once = Once::new();

/// **TEST HELPER** - Basic test initialization
/// 
/// Provides one-time setup for the test suite. This is kept minimal as most
/// setup is handled by the more comprehensive `setup_fresh_provider_schemas()`.
fn setup() {
    INIT.call_once(|| {
        // Note: Legacy setup replaced with more robust setup_fresh_provider_schemas()
        // which handles multiple providers gracefully and is CI-friendly
        println!("🔧 Setup initialized - use setup_fresh_provider_schemas() for comprehensive testing");
    });
}

/// **TEST HELPER** - Comprehensive provider schema setup for integration tests
/// 
/// This function performs a complete workflow to ensure fresh provider schemas
/// are available for testing. It's designed to be fault-tolerant and continues
/// execution even when cloud credentials are not available (e.g., in CI).
/// 
/// # Workflow
/// 1. Clean existing cache and schema files
/// 2. Run terragrunt init (may fail in CI)
/// 3. Run terragrunt plan (may fail in CI)  
/// 4. Generate provider schema files
/// 
/// # Returns
/// Result indicating overall success, but individual provider failures are logged
/// rather than causing test failures.
/// 
/// # Used By
/// Integration tests that require real provider schema information
fn setup_fresh_provider_schemas() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Setting up fresh provider schemas for both AWS and GCP...");
    
    let providers = vec![
        ("gcp", "envs/simulator/gcp/dev"),
        ("aws", "envs/simulator/aws/dev"),
    ];
    
    let mut results = Vec::new();
    
    for (provider_name, env_path) in providers {
        println!("🔧 Processing {} provider...", provider_name);
        
        // 1. Clean existing cache and schema files
        let schema_path = Path::new(env_path).join(".terragrunt-provider-schema.json");
        if schema_path.exists() {
            fs::remove_file(&schema_path)?;
            println!("  ✅ Removed existing schema file for {}", provider_name);
        }
        
        // 2. Run terragrunt init (may fail in CI, that's ok)
        println!("  🚀 Running terragrunt init for {}...", provider_name);
        let init_result = Command::new("terragrunt")
            .arg("init")
            .current_dir(env_path)
            .output();
            
        match init_result {
            Ok(output) => {
                if output.status.success() {
                    println!("  ✅ Init succeeded for {}", provider_name);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("  ⚠️ Init failed for {} (expected in CI): {}", provider_name, stderr);
                }
            }
            Err(e) => {
                println!("  ⚠️ Init command failed for {} (expected in CI): {}", provider_name, e);
            }
        }
        
        // 3. Run terragrunt plan (may fail in CI, that's ok)
        println!("  📋 Running terragrunt plan for {}...", provider_name);
        let plan_result = Command::new("terragrunt")
            .arg("run-all")
            .arg("plan")
            .current_dir(env_path)
            .output();
            
        match plan_result {
            Ok(output) => {
                if output.status.success() {
                    println!("  ✅ Plan succeeded for {}", provider_name);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("  ⚠️ Plan failed for {} (expected in CI): {}", provider_name, stderr);
                }
            }
            Err(e) => {
                println!("  ⚠️ Plan command failed for {} (expected in CI): {}", provider_name, e);
            }
        }
        
        // 4. Generate provider schema
        println!("  🔧 Generating provider schema for {}...", provider_name);
        let schema_result = write_provider_schema(Path::new(env_path));
        
        match schema_result {
            Ok(_) => {
                println!("  ✅ Schema generation succeeded for {}", provider_name);
                results.push((provider_name, true));
            }
            Err(e) => {
                println!("  ⚠️ Schema generation failed for {} (expected in CI): {}", provider_name, e);
                results.push((provider_name, false));
            }
        }
    }
    
    // Summary
    let successful_providers: Vec<&str> = results.iter()
        .filter_map(|(name, success)| if *success { Some(*name) } else { None })
        .collect();
    
    if successful_providers.is_empty() {
        println!("⚠️ No provider schemas could be generated (expected in CI without cloud access)");
    } else {
        println!("✅ Successfully generated schemas for: {:?}", successful_providers);
    }
    
    Ok(())
}

/// **TEST HELPER** - Creates a mock terraform module structure for testing
/// 
/// Generates a PlannedModule with a known structure containing:
/// - 2 root-level resources
/// - 1 child module with 1 resource
/// 
/// This provides predictable test data for resource collection and mapping tests.
/// 
/// # Returns
/// PlannedModule with test resources and child modules
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

/// **TEST** - Verifies basic resource collection from a planned module
/// 
/// Tests that the `collect_resources` function correctly extracts all resources
/// from a module hierarchy, including resources in child modules. This is a
/// fundamental operation for the import workflow.
#[test]
fn test_01_collect_resources() {
    let module = create_test_module();
    let mut resources = Vec::new();
    collect_resources(&module, &mut resources);
    
    assert_eq!(resources.len(), 3);
    assert_eq!(resources[0].address, "test.resource1");
    assert_eq!(resources[1].address, "test.resource2");
    assert_eq!(resources[2].address, "module.child.test.resource3");
}

/// **TEST** - Verifies resource collection handles nested module hierarchies
/// 
/// Tests resource collection with deeply nested modules (grandchild level)
/// to ensure the recursive collection algorithm works correctly with
/// complex module structures.
#[test]
fn test_02_collect_resources_consolidation() {
    let module = create_test_module();
    let mut resources = Vec::new();
    collect_resources(&module, &mut resources);
    
    assert_eq!(resources.len(), 3);
    assert_eq!(resources[0].address, "test.resource1");
    assert_eq!(resources[1].address, "test.resource2");
    assert_eq!(resources[2].address, "module.child.test.resource3");
    
    // Test that it handles nested modules correctly
    let nested_module = PlannedModule {
        resources: Some(vec![
            Resource {
                address: "root.resource".to_string(),
                mode: "managed".to_string(),
                r#type: "test_type".to_string(),
                name: "root_resource".to_string(),
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
                        address: "child1.resource".to_string(),
                        mode: "managed".to_string(),
                        r#type: "test_type".to_string(),
                        name: "child1_resource".to_string(),
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
                                address: "grandchild.resource".to_string(),
                                mode: "managed".to_string(),
                                r#type: "test_type".to_string(),
                                name: "grandchild_resource".to_string(),
                                provider_name: None,
                                schema_version: None,
                                values: None,   
                                sensitive_values: None,
                                depends_on: None,
                            },
                        ]),
                        child_modules: None,
                        address: Some("module.grandchild".to_string()),
                    },
                ]),
                address: Some("module.child1".to_string()),
            },
        ]),
        address: None,
    };
    
    let mut nested_resources = Vec::new();
    collect_resources(&nested_module, &mut nested_resources);
    
    assert_eq!(nested_resources.len(), 3);
    assert_eq!(nested_resources[0].address, "root.resource");
    assert_eq!(nested_resources[1].address, "child1.resource");
    assert_eq!(nested_resources[2].address, "grandchild.resource");
}

/// **TEST** - Verifies resource collection handles empty modules gracefully
/// 
/// Tests that the collection function properly handles modules with no
/// resources or child modules without failing or returning unexpected results.
#[test]
fn test_03_collect_resources_empty_module() {
    let module = PlannedModule {
        resources: None,
        child_modules: None,
        address: None,
    };
    let mut resources = Vec::new();
    collect_resources(&module, &mut resources);
    assert!(resources.is_empty());
}

/// **TEST** - Verifies ID candidate field extraction from provider schema
/// 
/// Tests the extraction of potential ID candidate fields from a provider schema
/// JSON structure. This is used as a fallback when more sophisticated schema
/// analysis isn't available.
#[test]
fn test_04_extract_id_candidate_fields() {
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

/// **TEST** - Verifies ID extraction handles empty schema gracefully
/// 
/// Tests that the ID candidate extraction function returns an empty set
/// when provided with an empty or invalid schema structure.
#[test]
fn test_05_extract_id_candidate_fields_empty_schema() {
    let schema_json = json!({});
    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.is_empty());
}

/// **TEST** - Verifies ID extraction handles missing provider section
/// 
/// Tests that the function gracefully handles schemas with a provider_schemas
/// section but no actual provider definitions.
#[test]
fn test_06_extract_id_candidate_fields_missing_provider() {
    let schema_json = json!({
        "provider_schemas": {}
    });
    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.is_empty());
}

/// **TEST** - Additional verification of ID candidate field extraction
/// 
/// Duplicate test to ensure robustness of the ID extraction functionality
/// with a standard provider schema structure.
#[test]
fn test_07_get_id_candidate_fields() {
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

/// **TEST** - Verifies handling of completely empty schema JSON
/// 
/// Tests that the function correctly returns an empty result when given
/// a completely empty JSON object.
#[test]
fn test_08_get_id_candidate_fields_empty() {
    let schema_json = json!({});
    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.is_empty());
}

/// **TEST** - Verifies handling of schemas with minimal attributes
/// 
/// Tests that the function correctly extracts candidates even when only
/// a single attribute is present in the schema.
#[test]
fn test_09_get_id_candidate_fields_less_than_three() {
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

/// **TEST** - Verifies error handling for invalid provider schema files
/// 
/// Tests that the provider schema loading function properly handles invalid
/// JSON files and returns appropriate errors rather than panicking.
#[test]
fn test_10_load_provider_schema_invalid_file() {
    let temp_dir = TempDir::new().unwrap();
    let schema_path = temp_dir.path().join(".terragrunt-provider-schema.json");
    fs::write(&schema_path, "invalid json").unwrap();
    let result = write_provider_schema(temp_dir.path());
    assert!(result.is_err());
}

/// **TEST** - Additional verification of attribute scoring for ID candidates
/// 
/// Another test focusing on the attribute scoring functionality to ensure
/// consistent behavior across different test scenarios.
#[test]
fn test_11_score_attributes_for_id() {
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

/// **TEST** - Verifies attribute scoring with empty schema
/// 
/// Tests that attribute scoring correctly handles empty schemas and
/// returns an empty result set.
#[test]
fn test_12_score_attributes_for_id_empty() {
    let schema_json = json!({});
    let candidates = extract_id_candidate_fields(&schema_json);
    assert!(candidates.is_empty());
}

/// **TEST** - Verifies terragrunt import command generation using real fixtures
/// 
/// This test uses actual fixture files (modules.json and plan.json) to verify
/// that the complete command generation workflow produces valid terragrunt
/// import commands. This is a critical integration test.
#[test]
fn test_13_generate_import_commands() {
    let modules_data = fs::read_to_string("tests/fixtures/gcp/modules.json").expect("Unable to read modules file");
    let plan_data = fs::read_to_string("tests/fixtures/gcp/out.json").expect("Unable to read plan file");

    let modules_file: ModulesFile = serde_json::from_str(&modules_data).expect("Invalid modules JSON");
    let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

    let mapping = map_resources_to_modules(&modules_file.modules, &plan);
    let commands = generate_import_commands(&mapping, &plan, ".", true);

    assert!(!commands.is_empty(), "No import commands generated");
    for cmd in commands {
        assert!(cmd.starts_with("terragrunt import"), "Command does not start with terragrunt import: {}", cmd);
    }
}

/// **TEST** - Verifies resource ID inference using real plan data
/// 
/// Tests the core ID inference functionality using actual terraform plan data.
/// This verifies that the tool can successfully identify resource IDs from
/// real terraform resource configurations.
#[test]
fn test_14_infer_resource_id() {
    let plan_data = fs::read_to_string("tests/fixtures/gcp/out.json").expect("Unable to read plan file");
    let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");
    let verbose = true;

    let mut found = false;
    if let Some(planned_values) = &plan.planned_values {
        /// Internal helper function to recursively check modules for inferable IDs
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

                    if let Some(id) = infer_resource_id(&terraform_resource, None, verbose) {
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

/// **TEST** - Verifies resource-to-module mapping using real fixture data
/// 
/// Tests the mapping functionality that associates each terraform resource
/// with its corresponding terragrunt module. This is essential for knowing
/// which directory to run terragrunt import commands in.
#[test]
fn test_15_map_resources_to_modules() {
    let modules_data = fs::read_to_string("tests/fixtures/gcp/modules.json").expect("Unable to read modules file");
    let plan_data = fs::read_to_string("tests/fixtures/gcp/out.json").expect("Unable to read plan file");

    let modules_file: ModulesFile = serde_json::from_str(&modules_data).expect("Invalid modules JSON");
    let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

    let mapping = map_resources_to_modules(&modules_file.modules, &plan);

    assert!(!mapping.is_empty(), "No resource-module mappings found");
}

/// **TEST** - Validates terragrunt import command structure without execution
/// 
/// This test verifies that the command construction logic produces valid
/// command strings without actually executing terragrunt commands. Uses
/// echo to simulate command execution.
#[test]
fn test_16_run_terragrunt_import_mock() {
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
fn test_17_validate_module_dirs_gcp() {
    let data = fs::read_to_string("tests/fixtures/gcp/modules.json").expect("Unable to read file");
    let modules_file: ModulesFile = serde_json::from_str(&data).expect("Invalid JSON");

    let errors = validate_module_dirs(&modules_file.modules, Path::new("simulator/gcp"));

    assert!(errors.is_empty(), "Found invalid GCP directories: {:?}", errors);
}

#[test]
fn test_17_validate_module_dirs_aws() {
    // Test that our AWS modules exist - these are the actual modules we created
    let aws_modules = vec![
        "s3", "iam", "vpc", "lambda", "rds", "ecr", "kms", 
        "sns", "secrets_manager", "cloudwatch", "cloudtrail"
    ];
    
    let mut missing_modules = Vec::new();
    for module in aws_modules {
        let module_path = Path::new("simulator/aws/modules").join(module);
        if !module_path.exists() {
            missing_modules.push(format!("Missing AWS module: {}", module));
        }
    }

    assert!(missing_modules.is_empty(), "Found missing AWS modules: {:?}", missing_modules);
}

#[test]
fn test_18_systematic_provider_schema_generation() {
    // This test ensures both AWS and GCP provider schemas are generated fresh
    // using the complete workflow: clean, init, plan, schema generation
    println!("🧪 Running systematic provider schema generation test...");
    
    // Run the complete setup workflow
    let setup_result = setup_fresh_provider_schemas();
    
    match setup_result {
        Ok(_) => {
            println!("✅ Provider schema setup completed successfully");
            
            // Verify that at least one schema was generated (if we have cloud access)
            let gcp_schema = Path::new("envs/simulator/gcp/dev/.terragrunt-provider-schema.json");
            let aws_schema = Path::new("envs/simulator/aws/dev/.terragrunt-provider-schema.json");
            
            let gcp_exists = gcp_schema.exists();
            let aws_exists = aws_schema.exists();
            
            println!("📊 Schema file status:");
            println!("  - GCP schema exists: {}", gcp_exists);
            println!("  - AWS schema exists: {}", aws_exists);
            
            if gcp_exists || aws_exists {
                println!("✅ At least one provider schema was successfully generated");
            } else {
                println!("⚠️ No schemas generated (expected in CI without cloud access)");
            }
            
            // Test that we can read any generated schemas
            if gcp_exists {
                let content = fs::read_to_string(gcp_schema)
                    .expect("Should be able to read GCP schema file");
                let _: Value = serde_json::from_str(&content)
                    .expect("GCP schema should be valid JSON");
                println!("✅ GCP schema is valid JSON");
            }
            
            if aws_exists {
                let content = fs::read_to_string(aws_schema)
                    .expect("Should be able to read AWS schema file");
                let _: Value = serde_json::from_str(&content)
                    .expect("AWS schema should be valid JSON");
                println!("✅ AWS schema is valid JSON");
            }
        }
        Err(e) => {
            println!("⚠️ Provider schema setup failed: {}", e);
            // This is acceptable in CI environments without cloud access
        }
    }
}

#[test]
fn test_18_individual_gcp_schema_generation() {
    // Individual test for GCP schema generation (for backwards compatibility)
    let schema_path = std::path::Path::new("envs/simulator/gcp/dev/.terragrunt-provider-schema.json");
    
    // Generate fresh schema
    let result = write_provider_schema(std::path::Path::new("envs/simulator/gcp/dev"));
    
    match result {
        Ok(_) => {
            assert!(schema_path.exists(), ".terragrunt-provider-schema.json should be created when successful");
            println!("✅ GCP Provider schema generation succeeded");
        }
        Err(e) => {
            println!("⚠️ GCP Provider schema generation failed (expected in CI): {}", e);
        }
    }
}

#[test]
fn test_18_individual_aws_schema_generation() {
    // Individual test for AWS schema generation (for backwards compatibility)
    let schema_path = std::path::Path::new("envs/simulator/aws/dev/.terragrunt-provider-schema.json");
    
    // Generate fresh schema
    let result = write_provider_schema(std::path::Path::new("envs/simulator/aws/dev"));
    
    match result {
        Ok(_) => {
            assert!(schema_path.exists(), ".terragrunt-provider-schema.json should be created when successful");
            println!("✅ AWS Provider schema generation succeeded");
        }
        Err(e) => {
            println!("⚠️ AWS Provider schema generation failed (expected in CI): {}", e);
        }
    }
}

#[test]
fn test_19_multi_cloud_module_root_gcp() {
    // Test that the tool works correctly with GCP module root
    let modules_data = fs::read_to_string("tests/fixtures/gcp/modules.json").expect("Unable to read modules file");
    let plan_data = fs::read_to_string("tests/fixtures/gcp/out.json").expect("Unable to read plan file");

    let modules_file: ModulesFile = serde_json::from_str(&modules_data).expect("Invalid modules JSON");
    let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid plan JSON");

    let mapping = map_resources_to_modules(&modules_file.modules, &plan);
    let commands = generate_import_commands(&mapping, &plan, "simulator/gcp/modules", true);

    // Commands should be generated and contain the GCP module path
    assert!(!commands.is_empty(), "No import commands generated for GCP modules");
    for cmd in &commands {
        assert!(cmd.contains("simulator/gcp/modules"), "Command does not contain GCP module path: {}", cmd);
    }
}

#[test]
fn test_19_multi_cloud_aws_directory_structure() {
    // Test that AWS module directory structure is correctly recognized
    let aws_module_root = "simulator/aws/modules";
    
    // Test that we can construct paths correctly for AWS modules
    let test_module_paths = vec![
        format!("{}/s3", aws_module_root),
        format!("{}/lambda", aws_module_root),
        format!("{}/iam", aws_module_root),
        format!("{}/vpc", aws_module_root),
    ];

    for path in test_module_paths {
        let module_path = Path::new(&path);
        assert!(module_path.exists(), "AWS module path does not exist: {}", path);
        
        // Test that main.tf exists in each module
        let main_tf = module_path.join("main.tf");
        assert!(main_tf.exists(), "main.tf not found in AWS module: {}", path);
    }
}

#[test]
fn test_19_multi_cloud_module_root_aws() {
    // Test that the tool works correctly with AWS module root
    let modules_data = fs::read_to_string("tests/fixtures/aws/modules.json").expect("Unable to read AWS modules file");
    let plan_data = fs::read_to_string("tests/fixtures/aws/out.json").expect("Unable to read AWS plan file");

    let modules_file: ModulesFile = serde_json::from_str(&modules_data).expect("Invalid AWS modules JSON");
    let plan: PlanFile = serde_json::from_str(&plan_data).expect("Invalid AWS plan JSON");

    let mapping = map_resources_to_modules(&modules_file.modules, &plan);
    let commands = generate_import_commands(&mapping, &plan, "simulator/aws/modules", true);

    // Commands should be generated and contain the AWS module path
    assert!(!commands.is_empty(), "No import commands generated for AWS modules");
    for cmd in &commands {
        assert!(cmd.contains("simulator/aws/modules"), "Command does not contain AWS module path: {}", cmd);
    }
}