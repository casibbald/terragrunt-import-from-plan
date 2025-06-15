//! # Provider-Specific ID Scoring Strategies Module
//! 
//! This module implements intelligent, provider-specific scoring strategies for identifying
//! the best resource ID candidates. Each cloud provider has unique naming conventions and
//! identifier patterns, so provider-specific strategies significantly improve ID inference
//! accuracy over generic approaches.
//! 
//! ## Key Features
//! 
//! - **Provider-Specific Intelligence**: Tailored scoring for Google Cloud, Azure, AWS
//! - **Schema-Driven Analysis**: Uses rich attribute metadata for intelligent scoring
//! - **Automatic Provider Detection**: Detects provider from resource type names
//! - **Extensible Architecture**: Easy to add new provider strategies
//! - **Fallback Support**: Generic strategy for unknown providers
//! 
//! ## Supported Providers
//! 
//! - **Google Cloud Platform**: Optimized for GCP resource naming patterns
//! - **Microsoft Azure**: Tailored for Azure resource conventions
//! - **Generic/Default**: Universal strategy for unknown providers
//! - **AWS**: Currently uses generic strategy (room for future enhancement)
//! 
//! ## Usage Patterns
//! 
//! 1. Auto-detect provider using `detect_provider_from_resource_type()`
//! 2. Create appropriate strategy using `create_scoring_strategy()`
//! 3. Use strategy to score attributes for ID candidate selection
//! 
//! ## Scoring Intelligence
//! 
//! Each strategy combines:
//! - **Name Pattern Analysis**: Provider-specific field naming conventions
//! - **Schema Metadata**: Required/computed fields, types, descriptions
//! - **Resource-Specific Logic**: Tailored scoring for specific resource types
//! - **Description Analysis**: Intelligent parsing of attribute descriptions

use serde_json::Value;
use super::traits::{IdScoringStrategy, ProviderType};
use crate::schema::metadata::AttributeMetadata;

/// Google Cloud Platform specific ID scoring strategy
/// 
/// This strategy implements GCP-specific intelligence for identifying the best resource
/// ID candidates. It understands GCP naming conventions, resource patterns, and the
/// importance of fields like `self_link` which are unique to Google Cloud.
/// 
/// # Key GCP-Specific Features
/// - Prioritizes `self_link` fields (GCP's unique self-referencing URLs)
/// - Understands resource-specific naming (e.g., `repository_id` for artifact registries)
/// - Optimized scoring for common GCP services (Compute, Storage, BigQuery, etc.)
/// - Intelligent handling of GCP's computed ID fields
/// 
/// # Scoring Priorities
/// 1. **Exact ID fields**: `id`, `self_link` (95-100 points)
/// 2. **Resource names**: `name`, `bucket`, `instance_id` (85-92 points)
/// 3. **Suffix patterns**: `*_id`, `*_name` (55-80 points)
/// 4. **Location fields**: `project`, `region`, `zone` (45-70 points)
/// 5. **Schema metadata bonuses**: Required/computed fields get additional points
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
            _name if definition.get("required").and_then(|v| v.as_bool()).unwrap_or(false) => 55.0,
            
            // String types are generally better for IDs
            _name if definition.get("type").and_then(|v| v.as_str()) == Some("string") => 40.0,
            
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
/// 
/// This strategy implements Azure-specific intelligence for identifying the best resource
/// ID candidates. It understands Azure naming conventions, resource group patterns, and
/// the importance of fields like `resource_id` and `fqdn` which are significant in Azure.
/// 
/// # Key Azure-Specific Features
/// - Prioritizes `resource_id` fields (Azure's resource identifiers)
/// - Understands `fqdn` importance for network and DNS resources
/// - Optimized for Azure naming patterns (`*_name`, resource groups)
/// - Intelligent handling of Azure location and subscription concepts
/// 
/// # Scoring Priorities
/// 1. **Exact ID fields**: `id`, `resource_id` (95-100 points)
/// 2. **Resource names**: `name`, `fqdn` (85-90 points)
/// 3. **Suffix patterns**: `*_id`, `*_name` (75-80 points)
/// 4. **Azure concepts**: `resource_group_name`, `location` (65-70 points)
/// 5. **Schema metadata bonuses**: Required/computed fields get additional points
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
            _name if definition.get("computed").and_then(|v| v.as_bool()).unwrap_or(false) => 45.0,
            _name if definition.get("required").and_then(|v| v.as_bool()).unwrap_or(false) => 55.0,
            _name if definition.get("type").and_then(|v| v.as_str()) == Some("string") => 40.0,
            
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
/// 
/// This strategy provides universal scoring logic that works across different cloud
/// providers. It focuses on common naming patterns and universal concepts that
/// apply regardless of the specific provider being used.
/// 
/// # Key Universal Features
/// - Provider-agnostic scoring based on universal patterns
/// - Emphasis on common field names (`id`, `name`, `*_id` patterns)
/// - Generic handling of computed and required fields
/// - Fallback strategy when provider-specific logic isn't available
/// 
/// # Scoring Priorities
/// 1. **Universal ID fields**: `id`, `name` (85-90 points)
/// 2. **Common patterns**: `*_id`, `*_name`, `identifier` (75-80 points)
/// 3. **Self-referencing**: `*link*`, `*url*`, `*self*` (70 points)
/// 4. **Location fields**: `region`, `location`, `zone` (60 points)
/// 5. **Schema metadata bonuses**: Required/computed fields get additional points
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
            _name if definition.get("computed").and_then(|v| v.as_bool()).unwrap_or(false) => 50.0,
            
            // Required fields - might be essential identifiers
            _name if definition.get("required").and_then(|v| v.as_bool()).unwrap_or(false) => 55.0,
            
            // String types - better for human-readable IDs
            _name if definition.get("type").and_then(|v| v.as_str()) == Some("string") => 40.0,
            
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

/// Creates an appropriate ID scoring strategy based on the provider type
/// 
/// This factory function returns the optimal scoring strategy for a given cloud provider.
/// Each provider has unique naming conventions and identifier patterns, so using the
/// correct provider-specific strategy significantly improves ID inference accuracy.
/// 
/// # Arguments
/// * `provider_type` - The cloud provider type to create a strategy for
/// 
/// # Returns
/// Box containing the appropriate IdScoringStrategy implementation
/// 
/// # Supported Providers
/// - **GoogleCloud**: Returns GoogleCloudScoringStrategy with GCP-specific logic
/// - **Azure**: Returns AzureScoringStrategy with Azure-specific logic
/// - **AWS**: Currently returns DefaultScoringStrategy (room for future enhancement)
/// - **Generic**: Returns DefaultScoringStrategy for unknown providers
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::scoring::strategies::{create_scoring_strategy, detect_provider_from_resource_type};
/// use terragrunt_import_from_plan::schema::metadata::AttributeMetadata;
/// 
/// // Auto-detect and create strategy
/// let provider = detect_provider_from_resource_type("google_storage_bucket");
/// let strategy = create_scoring_strategy(provider);
/// 
/// // Create sample metadata for testing
/// let metadata = AttributeMetadata {
///     required: true,
///     computed: false,
///     optional: false,
///     attr_type: "string".to_string(),
///     description: Some("Resource name".to_string()),
///     description_kind: None,
///     sensitive: None,
/// };
/// 
/// // Use strategy to score attributes
/// let score = strategy.score_attribute_with_metadata("name", &metadata, "google_storage_bucket");
/// println!("Attribute 'name' scored: {}", score);
/// ```
pub fn create_scoring_strategy(provider_type: ProviderType) -> Box<dyn IdScoringStrategy> {
    match provider_type {
        ProviderType::GoogleCloud => Box::new(GoogleCloudScoringStrategy),
        ProviderType::Azure => Box::new(AzureScoringStrategy),
        ProviderType::AWS => Box::new(DefaultScoringStrategy), // AWS implementation would go here
        ProviderType::Generic => Box::new(DefaultScoringStrategy),
    }
}

/// Automatically detects the cloud provider type from a terraform resource type string
/// 
/// This function analyzes the resource type naming convention to determine which cloud
/// provider it belongs to. This enables automatic selection of the appropriate
/// provider-specific scoring strategy for optimal ID inference.
/// 
/// # Arguments
/// * `resource_type` - Terraform resource type string (e.g., "google_storage_bucket")
/// 
/// # Returns
/// ProviderType enum indicating the detected cloud provider
/// 
/// # Detection Logic
/// - **GoogleCloud**: Resource types starting with "google_" or "gcp_"
/// - **Azure**: Resource types starting with "azurerm_" or "azure_"
/// - **AWS**: Resource types starting with "aws_"
/// - **Generic**: Any other resource type patterns
/// 
/// # Examples
/// ```
/// use terragrunt_import_from_plan::scoring::strategies::detect_provider_from_resource_type;
/// use terragrunt_import_from_plan::scoring::traits::ProviderType;
/// 
/// assert_eq!(detect_provider_from_resource_type("google_storage_bucket"), ProviderType::GoogleCloud);
/// assert_eq!(detect_provider_from_resource_type("azurerm_virtual_machine"), ProviderType::Azure);
/// assert_eq!(detect_provider_from_resource_type("aws_s3_bucket"), ProviderType::AWS);
/// assert_eq!(detect_provider_from_resource_type("custom_resource"), ProviderType::Generic);
/// ```
/// 
/// # Usage Pattern
/// ```no_run
/// use terragrunt_import_from_plan::scoring::strategies::{detect_provider_from_resource_type, create_scoring_strategy};
/// 
/// // Automatic provider detection and strategy creation
/// let provider = detect_provider_from_resource_type("google_compute_instance");
/// let strategy = create_scoring_strategy(provider);
/// 
/// // Now use the provider-optimized strategy for scoring
/// ```
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