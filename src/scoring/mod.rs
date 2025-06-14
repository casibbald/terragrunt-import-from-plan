pub mod strategies;
pub mod traits;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::metadata::AttributeMetadata;
    use strategies::*;
    use traits::*;
    use std::collections::HashMap;

    /// Helper function to create test metadata
    fn create_test_metadata(required: bool, computed: bool, attr_type: &str, description: Option<&str>) -> AttributeMetadata {
        AttributeMetadata {
            required,
            computed,
            optional: !required && !computed,
            attr_type: attr_type.to_string(),
            description: description.map(|s| s.to_string()),
            description_kind: Some("plain".to_string()),
            sensitive: None,
        }
    }

    #[test]
    fn test_google_cloud_schema_driven_scoring() {
        let strategy = GoogleCloudScoringStrategy;
        
        // Test repository_id for artifact registry - should score very high
        let repo_id_metadata = create_test_metadata(true, false, "string", Some("The repository ID"));
        let repo_id_score = strategy.score_attribute_with_metadata(
            "repository_id", 
            &repo_id_metadata, 
            "google_artifact_registry_repository"
        );
        
        // Test name for artifact registry - should score lower than repository_id
        let name_metadata = create_test_metadata(true, false, "string", Some("The repository name"));
        let name_score = strategy.score_attribute_with_metadata(
            "name", 
            &name_metadata, 
            "google_artifact_registry_repository"
        );
        
        println!("🎯 Artifact Registry Scoring:");
        println!("  repository_id: {:.1}", repo_id_score);
        println!("  name: {:.1}", name_score);
        
        // repository_id should score higher than name for artifact registries
        assert!(repo_id_score > name_score, 
            "repository_id ({:.1}) should score higher than name ({:.1}) for artifact registries", 
            repo_id_score, name_score);
        
        // Both should be over 80 (high confidence)
        assert!(repo_id_score > 80.0, "repository_id should be high confidence");
        assert!(name_score > 70.0, "name should still be decent confidence");
    }

    #[test]
    fn test_required_vs_optional_scoring() {
        let strategy = GoogleCloudScoringStrategy;
        
        // Required field should score higher than optional
        let required_metadata = create_test_metadata(true, false, "string", None);
        let optional_metadata = create_test_metadata(false, false, "string", None);
        
        let required_score = strategy.score_attribute_with_metadata(
            "test_field", 
            &required_metadata, 
            "google_storage_bucket"
        );
        
        let optional_score = strategy.score_attribute_with_metadata(
            "test_field", 
            &optional_metadata, 
            "google_storage_bucket"
        );
        
        println!("🎯 Required vs Optional:");
        println!("  required: {:.1}", required_score);
        println!("  optional: {:.1}", optional_score);
        
        assert!(required_score > optional_score, 
            "Required fields should score higher than optional fields");
    }

    #[test]
    fn test_computed_field_bonus() {
        let strategy = GoogleCloudScoringStrategy;
        
        // Computed field should get bonus points
        let computed_metadata = create_test_metadata(false, true, "string", None);
        let regular_metadata = create_test_metadata(false, false, "string", None);
        
        let computed_score = strategy.score_attribute_with_metadata(
            "test_field", 
            &computed_metadata, 
            "google_storage_bucket"
        );
        
        let regular_score = strategy.score_attribute_with_metadata(
            "test_field", 
            &regular_metadata, 
            "google_storage_bucket"
        );
        
        println!("🎯 Computed vs Regular:");
        println!("  computed: {:.1}", computed_score);
        println!("  regular: {:.1}", regular_score);
        
        assert!(computed_score > regular_score, 
            "Computed fields should score higher than regular fields");
    }

    #[test]
    fn test_description_based_scoring() {
        let strategy = GoogleCloudScoringStrategy;
        
        // Field with "unique identifier" in description should score higher
        let unique_metadata = create_test_metadata(
            false, false, "string", 
            Some("The unique identifier for this resource")
        );
        let regular_metadata = create_test_metadata(
            false, false, "string", 
            Some("A regular field for configuration")
        );
        
        let unique_score = strategy.score_attribute_with_metadata(
            "test_field", 
            &unique_metadata, 
            "google_storage_bucket"
        );
        
        let regular_score = strategy.score_attribute_with_metadata(
            "test_field", 
            &regular_metadata, 
            "google_storage_bucket"
        );
        
        println!("🎯 Description-based scoring:");
        println!("  unique identifier: {:.1}", unique_score);
        println!("  regular field: {:.1}", regular_score);
        
        assert!(unique_score > regular_score, 
            "Fields with 'unique identifier' in description should score higher");
    }

    #[test]
    fn test_resource_specific_overrides() {
        let strategy = GoogleCloudScoringStrategy;
        
        // Test bucket name for storage - should get resource-specific bonus
        let name_metadata = create_test_metadata(true, false, "string", None);
        
        let storage_name_score = strategy.score_attribute_with_metadata(
            "name", 
            &name_metadata, 
            "google_storage_bucket"
        );
        
        let generic_name_score = strategy.score_attribute_with_metadata(
            "name", 
            &name_metadata, 
            "google_some_other_resource"
        );
        
        println!("🎯 Resource-specific overrides:");
        println!("  storage bucket name: {:.1}", storage_name_score);
        println!("  generic resource name: {:.1}", generic_name_score);
        
        assert!(storage_name_score > generic_name_score, 
            "Storage bucket name should score higher due to resource-specific logic");
    }

    #[test]
    fn test_top_candidates_with_metadata() {
        let strategy = GoogleCloudScoringStrategy;
        
        // Create a map of attributes with different characteristics
        let mut attributes = HashMap::new();
        
        attributes.insert("id".to_string(), 
            create_test_metadata(false, true, "string", Some("Auto-generated ID")));
        attributes.insert("name".to_string(), 
            create_test_metadata(true, false, "string", Some("The resource name")));
        attributes.insert("repository_id".to_string(), 
            create_test_metadata(true, false, "string", Some("The repository identifier")));
        attributes.insert("location".to_string(), 
            create_test_metadata(true, false, "string", Some("The location")));
        attributes.insert("labels".to_string(), 
            create_test_metadata(false, false, "map", Some("Resource labels")));
        
        let top_candidates = strategy.get_top_candidates_with_metadata(
            "google_artifact_registry_repository", 
            &attributes, 
            3
        );
        
        println!("🎯 Top 3 candidates for artifact registry:");
        for (i, candidate) in top_candidates.iter().enumerate() {
            let score = strategy.score_attribute_with_metadata(
                candidate, 
                &attributes[candidate], 
                "google_artifact_registry_repository"
            );
            println!("  {}. {} ({:.1})", i + 1, candidate, score);
        }
        
        // repository_id should be the top candidate for artifact registries
        assert_eq!(top_candidates[0], "repository_id", 
            "repository_id should be the top candidate for artifact registries");
        
        // Should return exactly 3 candidates
        assert_eq!(top_candidates.len(), 3);
    }

    #[test]
    fn test_backward_compatibility() {
        let strategy = GoogleCloudScoringStrategy;
        
        // The old method should still work
        use serde_json::json;
        let definition = json!({
            "type": "string",
            "required": true,
            "computed": false
        });
        
        let old_score = strategy.score_attribute("name", &definition, "google_storage_bucket");
        
        // Should get a reasonable score using the old method
        assert!(old_score > 80.0, "Backward compatibility should still provide good scoring");
    }
} 