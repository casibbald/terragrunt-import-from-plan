use std::path::Path;
use terragrunt_import_from_plan::{SchemaManager, AttributeMetadata, ResourceAttributeMap};

/// Test that we can load the real schema file and parse resource attributes
#[test]
fn test_schema_manager_parse_real_attributes() {
    let schema_dir = Path::new("envs/simulator/gcp/dev");
    
    // Skip test if schema file doesn't exist (e.g., in CI without GCP access)
    if !schema_dir.join(".terragrunt-provider-schema.json").exists() {
        println!("⚠️ Skipping schema integration test - .terragrunt-provider-schema.json not found");
        return;
    }

    let mut schema_manager = SchemaManager::new(schema_dir);
    
    // Load the real schema
    schema_manager.load_or_generate_schema()
        .expect("Should be able to load schema from existing file");

    // Test parsing google_storage_bucket attributes
    let bucket_attributes = schema_manager.parse_resource_attributes("google_storage_bucket")
        .expect("Should be able to parse google_storage_bucket attributes");

    // Verify we have the expected attributes
    assert!(bucket_attributes.contains_key("name"), "Should have 'name' attribute");
    assert!(bucket_attributes.contains_key("id"), "Should have 'id' attribute");
    assert!(bucket_attributes.contains_key("location"), "Should have 'location' attribute");

    // Check the metadata for the 'name' attribute
    let name_metadata = &bucket_attributes["name"];
    assert_eq!(name_metadata.attr_type, "string");
    assert!(name_metadata.required, "name should be required for storage buckets");
    
    // Check the metadata for the 'id' attribute  
    let id_metadata = &bucket_attributes["id"];
    assert_eq!(id_metadata.attr_type, "string");
    assert!(id_metadata.computed, "id should be computed for storage buckets");

    println!("✅ Successfully parsed {} attributes for google_storage_bucket", bucket_attributes.len());
}

/// Test parsing google_artifact_registry_repository to verify repository_id is handled correctly
#[test]
fn test_artifact_registry_repository_parsing() {
    let schema_dir = Path::new("envs/simulator/gcp/dev");
    
    if !schema_dir.join(".terragrunt-provider-schema.json").exists() {
        println!("⚠️ Skipping artifact registry test - schema file not found");
        return;
    }

    let mut schema_manager = SchemaManager::new(schema_dir);
    schema_manager.load_or_generate_schema()
        .expect("Should load schema");

    let repo_attributes = schema_manager.parse_resource_attributes("google_artifact_registry_repository")
        .expect("Should parse artifact registry attributes");

    // Verify we have repository_id
    assert!(repo_attributes.contains_key("repository_id"), "Should have 'repository_id' attribute");
    assert!(repo_attributes.contains_key("name"), "Should have 'name' attribute");

    let repo_id_metadata = &repo_attributes["repository_id"];
    assert_eq!(repo_id_metadata.attr_type, "string");
    
    println!("✅ google_artifact_registry_repository has repository_id: required={}, computed={}", 
             repo_id_metadata.required, repo_id_metadata.computed);
}

/// Test the new get_id_candidate_attributes method
#[test] 
fn test_schema_driven_id_candidates() {
    let schema_dir = Path::new("envs/simulator/gcp/dev");
    
    if !schema_dir.join(".terragrunt-provider-schema.json").exists() {
        println!("⚠️ Skipping ID candidates test - schema file not found");
        return;
    }

    let mut schema_manager = SchemaManager::new(schema_dir);
    schema_manager.load_or_generate_schema()
        .expect("Should load schema");

    // Test with storage bucket
    let bucket_candidates = schema_manager.get_id_candidate_attributes("google_storage_bucket")
        .expect("Should get candidates for storage bucket");

    assert!(!bucket_candidates.is_empty(), "Should have some ID candidates");
    
    // Verify candidates are sorted by score (highest first)
    if bucket_candidates.len() > 1 {
        let first_score = bucket_candidates[0].1.calculate_base_score();
        let second_score = bucket_candidates[1].1.calculate_base_score();
        assert!(first_score >= second_score, "Candidates should be sorted by score");
    }

    for (name, metadata) in &bucket_candidates {
        println!("  📊 {} (score: {:.1}, required: {}, computed: {})", 
                 name, metadata.calculate_base_score(), metadata.required, metadata.computed);
    }

    // Test with artifact registry
    let repo_candidates = schema_manager.get_id_candidate_attributes("google_artifact_registry_repository")
        .expect("Should get candidates for artifact registry");

    let candidate_names: Vec<&String> = repo_candidates.iter().map(|(name, _)| name).collect();
    println!("🎯 Artifact Registry ID candidates: {:?}", candidate_names);
}

/// Test listing all available resource types from the schema
#[test]
fn test_list_resource_types() {
    let schema_dir = Path::new("envs/simulator/gcp/dev");
    
    if !schema_dir.join(".terragrunt-provider-schema.json").exists() {
        println!("⚠️ Skipping resource types test - schema file not found");
        return;
    }

    let mut schema_manager = SchemaManager::new(schema_dir);
    schema_manager.load_or_generate_schema()
        .expect("Should load schema");

    let resource_types = schema_manager.list_resource_types()
        .expect("Should list resource types");

    assert!(!resource_types.is_empty(), "Should have resource types");
    assert!(resource_types.contains(&"google_storage_bucket".to_string()), "Should include storage bucket");
    assert!(resource_types.contains(&"google_artifact_registry_repository".to_string()), "Should include artifact registry");

    println!("📋 Found {} resource types in schema", resource_types.len());
    
    // Print first 10 for verification
    for (i, rt) in resource_types.iter().take(10).enumerate() {
        println!("  {}. {}", i + 1, rt);
    }
    if resource_types.len() > 10 {
        println!("  ... and {} more", resource_types.len() - 10);
    }
}

/// Test that our scoring logic works correctly with real metadata
#[test]
fn test_metadata_scoring_logic() {
    // Test with a required string field (should score high)
    let required_string = AttributeMetadata {
        required: true,
        computed: false,
        optional: false,
        attr_type: "string".to_string(),
        description: Some("The unique identifier for this resource".to_string()),
        description_kind: Some("plain".to_string()),
        sensitive: None,
    };

    let score = required_string.calculate_base_score();
    // Base(30) + Required(15) + String(5) + "identifier"(8) = 58
    assert!(score >= 58.0, "Required string with 'identifier' description should score high: {}", score);

    // Test with computed field (should score medium-high)
    let computed_string = AttributeMetadata {
        required: false,
        computed: true,
        optional: true,
        attr_type: "string".to_string(),
        description: None,
        description_kind: None,
        sensitive: None,
    };

    let computed_score = computed_string.calculate_base_score();
    // Base(30) + Computed(10) + String(5) = 45
    assert!(computed_score >= 45.0, "Computed string should score medium-high: {}", computed_score);

    // Test with optional non-string (should score low)
    let optional_bool = AttributeMetadata {
        required: false,
        computed: false,
        optional: true,
        attr_type: "bool".to_string(),
        description: None,
        description_kind: None,
        sensitive: None,
    };

    let bool_score = optional_bool.calculate_base_score();
    // Base(30) only = 30
    assert_eq!(bool_score, 30.0, "Optional bool should have base score only");

    println!("✅ Scoring logic working: required_string={:.1}, computed_string={:.1}, optional_bool={:.1}", 
             score, computed_score, bool_score);
} 