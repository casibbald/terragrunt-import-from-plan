use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about a terraform resource attribute extracted from provider schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeMetadata {
    /// Whether this attribute is required when creating the resource
    pub required: bool,
    
    /// Whether this attribute is computed (auto-generated) by the provider
    pub computed: bool,
    
    /// Whether this attribute is optional (can be omitted)
    pub optional: bool,
    
    /// The type of the attribute ("string", "number", "bool", "list", "map", etc.)
    pub attr_type: String,
    
    /// Human-readable description of the attribute
    pub description: Option<String>,
    
    /// The kind of description ("plain", "markdown")
    pub description_kind: Option<String>,
    
    /// Whether this attribute is sensitive (contains secrets)
    pub sensitive: Option<bool>,
}

impl AttributeMetadata {
    /// Create a new AttributeMetadata from terraform schema JSON
    pub fn from_schema_value(value: &serde_json::Value) -> Result<Self, AttributeMetadataError> {
        let required = value.get("required").and_then(|v| v.as_bool()).unwrap_or(false);
        let computed = value.get("computed").and_then(|v| v.as_bool()).unwrap_or(false);
        let optional = value.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
        
        let attr_type = value
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
            
        let description = value
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let description_kind = value
            .get("description_kind")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        let sensitive = value.get("sensitive").and_then(|v| v.as_bool());
        
        Ok(AttributeMetadata {
            required,
            computed,
            optional,
            attr_type,
            description,
            description_kind,
            sensitive,
        })
    }
    
    /// Calculate a relevance score for ID inference
    /// Higher scores indicate better candidates for resource identification
    pub fn calculate_base_score(&self) -> f64 {
        let mut score = 30.0; // Base score
        
        // Prefer required fields - they're essential to the resource
        if self.required {
            score += 15.0;
        }
        
        // Computed fields are often auto-generated IDs
        if self.computed {
            score += 10.0;
        }
        
        // String types are generally better for human-readable IDs
        if self.attr_type == "string" {
            score += 5.0;
        }
        
        // Bonus for fields that mention identification in description
        if let Some(ref desc) = self.description {
            let desc_lower = desc.to_lowercase();
            if desc_lower.contains("identifier") || desc_lower.contains("unique") {
                score += 8.0;
            }
            if desc_lower.contains("name") || desc_lower.contains("id") {
                score += 5.0;
            }
        }
        
        score
    }
    
    /// Check if this attribute looks like it could be an ID field
    pub fn is_potential_id(&self) -> bool {
        // Must be a string type for most ID use cases
        if self.attr_type != "string" {
            return false;
        }
        
        // Either required or computed (not just optional metadata)
        self.required || self.computed
    }
}

/// Map of attribute names to their metadata for a specific resource type
pub type ResourceAttributeMap = HashMap<String, AttributeMetadata>;

/// Errors that can occur when parsing attribute metadata
#[derive(Debug, thiserror::Error)]
pub enum AttributeMetadataError {
    #[error("Missing required field in schema: {field}")]
    MissingField { field: String },
    
    #[error("Invalid type for field {field}: expected {expected}, got {actual}")]
    InvalidType {
        field: String,
        expected: String,
        actual: String,
    },
    
    #[error("Schema parsing error: {message}")]
    ParseError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_attribute_metadata_from_schema_value() {
        let schema = json!({
            "type": "string",
            "description": "The unique identifier for the resource",
            "description_kind": "plain",
            "required": true,
            "computed": false
        });

        let metadata = AttributeMetadata::from_schema_value(&schema).unwrap();
        assert_eq!(metadata.attr_type, "string");
        assert!(metadata.required);
        assert!(!metadata.computed);
        assert_eq!(metadata.description, Some("The unique identifier for the resource".to_string()));
    }

    #[test]
    fn test_calculated_base_score() {
        let metadata = AttributeMetadata {
            required: true,
            computed: false,
            optional: false,
            attr_type: "string".to_string(),
            description: Some("The unique identifier for this resource".to_string()),
            description_kind: Some("plain".to_string()),
            sensitive: None,
        };

        let score = metadata.calculate_base_score();
        // Base(30) + Required(15) + String(5) + ID in description(5) = 55
        assert!(score >= 55.0);
    }

    #[test]
    fn test_is_potential_id() {
        let good_candidate = AttributeMetadata {
            required: true,
            computed: false,
            optional: false,
            attr_type: "string".to_string(),
            description: None,
            description_kind: None,
            sensitive: None,
        };
        assert!(good_candidate.is_potential_id());

        let bad_candidate = AttributeMetadata {
            required: false,
            computed: false,
            optional: true,
            attr_type: "bool".to_string(),
            description: None,
            description_kind: None,
            sensitive: None,
        };
        assert!(!bad_candidate.is_potential_id());
    }
} 