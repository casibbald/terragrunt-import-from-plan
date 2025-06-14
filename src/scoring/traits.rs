use serde_json::Value;
use std::collections::HashMap;
use crate::schema::metadata::AttributeMetadata;

/// Supported provider types for specialized scoring strategies
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderType {
    GoogleCloud,
    Azure,
    AWS,
    Generic,
}

/// Trait for implementing ID scoring strategies for different providers
pub trait IdScoringStrategy {
    /// Score a single attribute based on how likely it is to be a useful ID field
    /// Higher scores indicate better candidates for resource IDs
    /// 
    /// # Arguments
    /// * `name` - The attribute name (e.g., "id", "name", "self_link")
    /// * `definition` - The schema definition for this attribute
    /// * `resource_type` - The type of the resource (e.g., "google_storage_bucket")
    /// 
    /// # Returns
    /// A score between 0.0 and 100.0, where 100.0 is the highest confidence
    fn score_attribute(&self, name: &str, definition: &Value, resource_type: &str) -> f64;

    /// Score a single attribute using rich schema metadata (preferred method)
    /// Higher scores indicate better candidates for resource IDs
    /// 
    /// # Arguments
    /// * `name` - The attribute name (e.g., "id", "name", "self_link")
    /// * `metadata` - Rich metadata from schema parsing
    /// * `resource_type` - The type of the resource (e.g., "google_storage_bucket")
    /// 
    /// # Returns
    /// A score between 0.0 and 100.0, where 100.0 is the highest confidence
    fn score_attribute_with_metadata(&self, name: &str, metadata: &AttributeMetadata, resource_type: &str) -> f64 {
        // Default implementation - convert metadata back to basic schema format for backward compatibility
        let mut definition = serde_json::Map::new();
        definition.insert("required".to_string(), serde_json::Value::Bool(metadata.required));
        definition.insert("computed".to_string(), serde_json::Value::Bool(metadata.computed));
        definition.insert("optional".to_string(), serde_json::Value::Bool(metadata.optional));
        definition.insert("type".to_string(), serde_json::Value::String(metadata.attr_type.clone()));
        
        if let Some(ref desc) = metadata.description {
            definition.insert("description".to_string(), serde_json::Value::String(desc.clone()));
        }
        
        self.score_attribute(name, &serde_json::Value::Object(definition), resource_type)
    }

    /// Score all attributes in a resource schema and return a ranked list
    /// 
    /// # Arguments
    /// * `resource_type` - The type of the resource
    /// * `attributes` - Map of attribute names to their schema definitions
    /// 
    /// # Returns
    /// A HashMap of attribute names to their scores, sorted by score (highest first)
    fn score_all_attributes(&self, resource_type: &str, attributes: &HashMap<String, Value>) -> HashMap<String, f64> {
        let mut scores = HashMap::new();
        
        for (name, definition) in attributes {
            let score = self.score_attribute(name, definition, resource_type);
            scores.insert(name.clone(), score);
        }
        
        scores
    }

    /// Score all attributes using rich metadata (preferred method)
    fn score_all_attributes_with_metadata(&self, resource_type: &str, attributes: &HashMap<String, AttributeMetadata>) -> HashMap<String, f64> {
        let mut scores = HashMap::new();
        
        for (name, metadata) in attributes {
            let score = self.score_attribute_with_metadata(name, metadata, resource_type);
            scores.insert(name.clone(), score);
        }
        
        scores
    }

    /// Get the top N candidate attributes for a resource type
    /// 
    /// # Arguments
    /// * `resource_type` - The type of the resource
    /// * `attributes` - Map of attribute names to their schema definitions
    /// * `limit` - Maximum number of candidates to return (default: 5)
    /// 
    /// # Returns
    /// A Vec of attribute names, ordered by score (highest first)
    fn get_top_candidates(&self, resource_type: &str, attributes: &HashMap<String, Value>, limit: usize) -> Vec<String> {
        let scores = self.score_all_attributes(resource_type, attributes);
        let mut scored_attrs: Vec<(String, f64)> = scores.into_iter().collect();
        scored_attrs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        scored_attrs
            .into_iter()
            .take(limit)
            .map(|(name, _)| name)
            .collect()
    }

    /// Get the top N candidate attributes using rich metadata (preferred method)
    fn get_top_candidates_with_metadata(&self, resource_type: &str, attributes: &HashMap<String, AttributeMetadata>, limit: usize) -> Vec<String> {
        let scores = self.score_all_attributes_with_metadata(resource_type, attributes);
        let mut scored_attrs: Vec<(String, f64)> = scores.into_iter().collect();
        scored_attrs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        scored_attrs
            .into_iter()
            .take(limit)
            .map(|(name, _)| name)
            .collect()
    }

    /// Get the provider type this strategy is optimized for
    fn provider_type(&self) -> ProviderType;

    /// Get a human-readable name for this strategy
    fn strategy_name(&self) -> &'static str;
} 