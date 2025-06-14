use serde_json::Value;
use super::traits::{IdScoringStrategy, ProviderType};
use crate::schema::metadata::AttributeMetadata;

/// Google Cloud Platform specific ID scoring strategy
pub struct GoogleCloudScoringStrategy;

impl IdScoringStrategy for GoogleCloudScoringStrategy {
    fn score_attribute(&self, name: &str, definition: &Value, resource_type: &str) -> f64 {
        // GCP-specific scoring logic (legacy method for backward compatibility)
        match name {
            // Highest priority - exact ID fields
            "id" => 100.0,
            "self_link" => 95.0, // GCP's unique self-referencing URLs
            
            // High priority - common GCP identifiers
            "name" => if resource_type.contains("bucket") || resource_type.contains("instance") {
                90.0
            } else {
                85.0
            },
            "bucket" => if resource_type.contains("storage") { 92.0 } else { 70.0 },
            "instance_id" => if resource_type.contains("compute") { 90.0 } else { 60.0 },
            "cluster_name" => if resource_type.contains("gke") || resource_type.contains("dataproc") { 88.0 } else { 60.0 },
            
            // Medium priority - GCP resource identifiers
            name if name.ends_with("_id") => 80.0,
            name if name.ends_with("_name") => 75.0,
            "project" => 70.0,
            "location" | "region" | "zone" => 65.0,
            
            // Lower priority - computed or metadata fields
            name if definition.get("computed").and_then(|v| v.as_bool()).unwrap_or(false) => {
                if name.contains("url") || name.contains("link") { 60.0 } else { 45.0 }
            },
            
            // Required fields might be identifying
            name if definition.get("required").and_then(|v| v.as_bool()).unwrap_or(false) => 55.0,
            
            // String types are generally better for IDs
            name if definition.get("type").and_then(|v| v.as_str()) == Some("string") => 40.0,
            
            // Default score for other attributes
            _ => 30.0,
        }
    }

    /// ✨ NEW: Schema-driven scoring using rich AttributeMetadata
    /// This replaces hardcoded rules with intelligent metadata analysis
    fn score_attribute_with_metadata(&self, name: &str, metadata: &AttributeMetadata, resource_type: &str) -> f64 {
        let mut score: f64 = 0.0;
        
        // 🎯 Base score from attribute name patterns (GCP-specific intelligence)
        score += match name {
            // Highest priority - exact ID fields
            "id" => 70.0,
            "self_link" => 75.0,                    // GCP's unique self-referencing URLs
            "name" => 65.0,
            
            // High priority - GCP-specific identifiers
            name if name.ends_with("_id") => 60.0,
            name if name.ends_with("_name") => 55.0,
            name if name.contains("identifier") => 58.0,
            
            // Medium priority - common GCP fields
            "project" => 50.0,
            "location" | "region" | "zone" => 45.0,
            "bucket" => 50.0,
            "instance_id" => 55.0,
            "cluster_name" => 52.0,
            "repository_id" => 65.0,                // Important for artifact registries
            
            // Self-referencing or link fields
            name if name.contains("url") || name.contains("link") => 40.0,
            
            // Default base score
            _ => 30.0,
        };
        
        // ✨ NEW: Schema metadata bonuses - this is the key enhancement!
        if metadata.required {
            score += 15.0;  // Required fields are essential to the resource
        }
        
        if metadata.computed {
            score += 10.0;  // Computed fields are often auto-generated IDs
        }
        
        if metadata.attr_type == "string" {
            score += 5.0;   // String types make good human-readable IDs
        }
        
        // ✨ NEW: Description-based intelligence
        if let Some(ref description) = metadata.description {
            let desc_lower = description.to_lowercase();
            if desc_lower.contains("unique") || desc_lower.contains("identifier") {
                score += 8.0;
            }
            if desc_lower.contains("id") || desc_lower.contains("name") {
                score += 5.0;
            }
        }
        
        // ✨ NEW: Resource-specific logic (intelligent overrides)
        score += match (resource_type, name) {
            // Artifact Registry: repository_id is the key identifier
            ("google_artifact_registry_repository", "repository_id") => 20.0,
            ("google_artifact_registry_repository", "name") => -10.0, // Downgrade name for registries
            
            // Storage: bucket name is critical
            ("google_storage_bucket", "name") => 15.0,
            ("google_storage_bucket", "bucket") => 10.0,
            
            // Compute: instance_id is preferred over name
            ("google_compute_instance", "instance_id") => 20.0,
            ("google_compute_instance", "name") => -5.0,
            
            // BigQuery: dataset_id and table_id take precedence
            ("google_bigquery_dataset", "dataset_id") => 18.0,
            ("google_bigquery_table", "table_id") => 18.0,
            
            // Cloud Functions: function name is key
            ("google_cloudfunctions_function", "name") => 12.0,
            ("google_cloudfunctions2_function", "name") => 12.0,
            
            // Pub/Sub: topic and subscription names
            ("google_pubsub_topic", "name") => 15.0,
            ("google_pubsub_subscription", "name") => 15.0,
            
            // Cloud SQL: instance name is critical
            ("google_sql_database_instance", "name") => 15.0,
            ("google_sql_database", "name") => 10.0, // Less critical for DB within instance
            
            // GKE: cluster name is essential
            ("google_container_cluster", "name") => 15.0,
            ("google_container_node_pool", "name") => 10.0,
            
            // Default: no resource-specific bonus
            _ => 0.0,
        };
        
        // Cap the score at 100.0
        if score > 100.0 { 100.0 } else { score }
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::GoogleCloud
    }

    fn strategy_name(&self) -> &'static str {
        "Google Cloud Platform (Schema-Driven)"
    }
}

/// Azure specific ID scoring strategy
pub struct AzureScoringStrategy;

impl IdScoringStrategy for AzureScoringStrategy {
    fn score_attribute(&self, name: &str, definition: &Value, resource_type: &str) -> f64 {
        // Azure-specific scoring logic (legacy method for backward compatibility)
        match name {
            // Highest priority - exact ID fields
            "id" => 100.0,
            
            // High priority - Azure-specific identifiers
            "name" => 90.0,
            "resource_id" => 95.0,
            "fqdn" => if resource_type.contains("dns") || resource_type.contains("network") { 88.0 } else { 70.0 },
            
            // Medium priority - Azure naming patterns
            name if name.ends_with("_id") => 80.0,
            name if name.ends_with("_name") => 75.0,
            "resource_group_name" => 70.0,
            "location" => 65.0,
            "subscription_id" => 60.0,
            
            // Lower priority - computed fields
            name if definition.get("computed").and_then(|v| v.as_bool()).unwrap_or(false) => 45.0,
            name if definition.get("required").and_then(|v| v.as_bool()).unwrap_or(false) => 55.0,
            name if definition.get("type").and_then(|v| v.as_str()) == Some("string") => 40.0,
            
            _ => 30.0,
        }
    }

    /// ✨ NEW: Schema-driven scoring for Azure resources
    fn score_attribute_with_metadata(&self, name: &str, metadata: &AttributeMetadata, resource_type: &str) -> f64 {
        let mut score: f64 = 0.0;
        
        // 🎯 Base score from attribute name patterns (Azure-specific intelligence)
        score += match name {
            // Highest priority - exact ID fields
            "id" => 90.0,
            "resource_id" => 95.0,
            "name" => 85.0,
            
            // High priority - Azure-specific identifiers
            name if name.ends_with("_id") => 80.0,
            name if name.ends_with("_name") => 75.0,
            "fqdn" => 78.0,
            
            // Medium priority - Azure naming patterns
            "resource_group_name" => 70.0,
            "location" => 65.0,
            "subscription_id" => 60.0,
            
            // Default base score
            _ => 50.0,
        };
        
        // ✨ Schema metadata bonuses
        if metadata.required {
            score += 15.0;
        }
        
        if metadata.computed {
            score += 10.0;
        }
        
        if metadata.attr_type == "string" {
            score += 5.0;
        }
        
        // Description-based intelligence
        if let Some(ref description) = metadata.description {
            let desc_lower = description.to_lowercase();
            if desc_lower.contains("unique") || desc_lower.contains("identifier") {
                score += 8.0;
            }
            if desc_lower.contains("id") || desc_lower.contains("name") {
                score += 5.0;
            }
        }
        
        // Azure resource-specific logic
        score += match (resource_type, name) {
            ("azurerm_storage_account", "name") => 15.0,
            ("azurerm_virtual_machine", "name") => 15.0,
            ("azurerm_resource_group", "name") => 18.0,
            ("azurerm_key_vault", "name") => 15.0,
            ("azurerm_sql_database", "name") => 12.0,
            ("azurerm_virtual_network", "name") => 12.0,
            _ => 0.0,
        };
        
        // Cap the score at 100.0
        if score > 100.0 { 100.0 } else { score }
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Azure
    }

    fn strategy_name(&self) -> &'static str {
        "Microsoft Azure (Schema-Driven)"
    }
}

/// Default/generic ID scoring strategy for unknown or multi-provider scenarios
pub struct DefaultScoringStrategy;

impl IdScoringStrategy for DefaultScoringStrategy {
    fn score_attribute(&self, name: &str, definition: &Value, _resource_type: &str) -> f64 {
        // Generic scoring logic that works across providers (legacy method)
        match name {
            // Universal highest priority
            "id" => 100.0,
            "name" => 90.0,
            
            // Common patterns across providers
            name if name.ends_with("_id") => 80.0,
            name if name.ends_with("_name") => 75.0,
            name if name.contains("identifier") => 78.0,
            
            // Self-referencing fields
            name if name.contains("self") || name.contains("link") || name.contains("url") => 70.0,
            
            // Location/region fields
            name if name.contains("region") || name.contains("location") || name.contains("zone") => 60.0,
            
            // Computed fields - might be generated IDs
            name if definition.get("computed").and_then(|v| v.as_bool()).unwrap_or(false) => 50.0,
            
            // Required fields - might be essential identifiers
            name if definition.get("required").and_then(|v| v.as_bool()).unwrap_or(false) => 55.0,
            
            // String types - better for human-readable IDs
            name if definition.get("type").and_then(|v| v.as_str()) == Some("string") => 40.0,
            
            _ => 30.0,
        }
    }

    /// ✨ NEW: Schema-driven scoring for generic providers
    fn score_attribute_with_metadata(&self, name: &str, metadata: &AttributeMetadata, _resource_type: &str) -> f64 {
        let mut score: f64 = 0.0;
        
        // 🎯 Base score from attribute name patterns (universal patterns)
        score += match name {
            // Universal highest priority
            "id" => 90.0,
            "name" => 85.0,
            
            // Common patterns across providers
            name if name.ends_with("_id") => 80.0,
            name if name.ends_with("_name") => 75.0,
            name if name.contains("identifier") => 78.0,
            
            // Self-referencing fields
            name if name.contains("self") || name.contains("link") || name.contains("url") => 70.0,
            
            // Location/region fields
            name if name.contains("region") || name.contains("location") || name.contains("zone") => 60.0,
            
            // Default base score
            _ => 50.0,
        };
        
        // ✨ Schema metadata bonuses
        if metadata.required {
            score += 15.0;
        }
        
        if metadata.computed {
            score += 10.0;
        }
        
        if metadata.attr_type == "string" {
            score += 5.0;
        }
        
        // Description-based intelligence
        if let Some(ref description) = metadata.description {
            let desc_lower = description.to_lowercase();
            if desc_lower.contains("unique") || desc_lower.contains("identifier") {
                score += 8.0;
            }
            if desc_lower.contains("id") || desc_lower.contains("name") {
                score += 5.0;
            }
        }
        
        // Cap the score at 100.0
        if score > 100.0 { 100.0 } else { score }
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Generic
    }

    fn strategy_name(&self) -> &'static str {
        "Default (Generic Schema-Driven)"
    }
}

/// Factory function to create the appropriate scoring strategy based on provider
pub fn create_scoring_strategy(provider_type: ProviderType) -> Box<dyn IdScoringStrategy> {
    match provider_type {
        ProviderType::GoogleCloud => Box::new(GoogleCloudScoringStrategy),
        ProviderType::Azure => Box::new(AzureScoringStrategy),
        ProviderType::AWS => Box::new(DefaultScoringStrategy), // AWS implementation would go here
        ProviderType::Generic => Box::new(DefaultScoringStrategy),
    }
}

/// Auto-detect provider type from resource type string
pub fn detect_provider_from_resource_type(resource_type: &str) -> ProviderType {
    if resource_type.starts_with("google_") || resource_type.starts_with("gcp_") {
        ProviderType::GoogleCloud
    } else if resource_type.starts_with("azurerm_") || resource_type.starts_with("azure_") {
        ProviderType::Azure
    } else if resource_type.starts_with("aws_") {
        ProviderType::AWS
    } else {
        ProviderType::Generic
    }
} 