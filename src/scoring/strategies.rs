use serde_json::Value;
use super::traits::{IdScoringStrategy, ProviderType};

/// Google Cloud Platform specific ID scoring strategy
pub struct GoogleCloudScoringStrategy;

impl IdScoringStrategy for GoogleCloudScoringStrategy {
    fn score_attribute(&self, name: &str, definition: &Value, resource_type: &str) -> f64 {
        // GCP-specific scoring logic
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

    fn provider_type(&self) -> ProviderType {
        ProviderType::GoogleCloud
    }

    fn strategy_name(&self) -> &'static str {
        "Google Cloud Platform"
    }
}

/// Azure specific ID scoring strategy
pub struct AzureScoringStrategy;

impl IdScoringStrategy for AzureScoringStrategy {
    fn score_attribute(&self, name: &str, definition: &Value, resource_type: &str) -> f64 {
        // Azure-specific scoring logic
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

    fn provider_type(&self) -> ProviderType {
        ProviderType::Azure
    }

    fn strategy_name(&self) -> &'static str {
        "Microsoft Azure"
    }
}

/// Default/generic ID scoring strategy for unknown or multi-provider scenarios
pub struct DefaultScoringStrategy;

impl IdScoringStrategy for DefaultScoringStrategy {
    fn score_attribute(&self, name: &str, definition: &Value, _resource_type: &str) -> f64 {
        // Generic scoring logic that works across providers
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

    fn provider_type(&self) -> ProviderType {
        ProviderType::Generic
    }

    fn strategy_name(&self) -> &'static str {
        "Default (Generic)"
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