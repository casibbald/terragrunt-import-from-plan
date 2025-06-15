//! # Terraform Plan Processing Module
//! 
//! This module provides functionality for working with terraform plans and provider schemas.
//! It includes structures for representing terraform plans, functions for loading and parsing
//! provider schemas, and intelligent scoring algorithms for identifying resource ID candidates.
//! 
//! ## Key Components
//! 
//! - **Plan Structures**: Represent terraform plan data with proper deserialization
//! - **Schema Loading**: Load and parse terraform provider schemas from JSON files
//! - **ID Scoring**: Intelligent algorithms to score attributes for ID suitability
//! - **Candidate Selection**: Extract top ID candidate fields based on scoring
//! 
//! ## Usage Patterns
//! 
//! 1. Use TerraformResource and related structs to represent plan data
//! 2. Load provider schemas using load_provider_schema()
//! 3. Score attributes using score_attributes_for_id()
//! 4. Get top candidates using get_id_candidate_fields()

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Top-level structure representing a terraform plan
/// 
/// This structure represents the overall terraform plan containing all planned
/// changes and resource information. It serves as the root container for plan data.
/// 
/// # Fields
/// - `planned_values`: Contains all the planned resource values and module structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformPlan {
    /// Contains all planned resource values and module hierarchy
    pub planned_values: PlannedValues,
}

/// Container for all planned values in a terraform plan
/// 
/// This structure contains the root module and all planned resource values.
/// It represents the state that terraform plans to achieve after applying changes.
/// 
/// # Fields
/// - `root_module`: The root module containing all resources and child modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedValues {
    /// The root module containing the complete resource hierarchy
    pub root_module: RootModule,
}

/// Represents the root module in a terraform plan
/// 
/// The root module is the top-level container for all resources and child modules
/// in a terraform configuration. It provides the entry point for traversing the
/// complete module hierarchy.
/// 
/// # Fields
/// - `child_modules`: Vector of child modules contained within this root module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootModule {
    /// Collection of child modules nested within the root module
    pub child_modules: Vec<ChildModule>,
}

/// Represents a child module within a terraform plan
/// 
/// Child modules represent nested terraform modules that contain their own
/// resources and configurations. Each child module can contain multiple resources
/// that will be managed as part of the overall terraform plan.
/// 
/// # Fields
/// - `resources`: Vector of terraform resources defined in this module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildModule {
    /// Collection of terraform resources defined within this module
    pub resources: Vec<TerraformResource>,
}

/// Represents a single terraform resource in a plan
/// 
/// This structure contains all the essential information about a terraform resource
/// including its address, type, configuration values, and metadata. It serves as
/// the fundamental unit for resource processing and import operations.
/// 
/// # Fields
/// - `address`: Full terraform address (e.g., "module.vpc.aws_vpc.main")
/// - `mode`: Resource mode (typically "managed" for regular resources)
/// - `type`: Resource type (e.g., "aws_vpc", "google_storage_bucket")
/// - `name`: Resource name within the module
/// - `values`: Optional configuration values for the resource
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::plan::TerraformResource;
/// 
/// let resource = TerraformResource {
///     address: "module.vpc.aws_vpc.main".to_string(),
///     mode: "managed".to_string(),
///     r#type: "aws_vpc".to_string(),
///     name: "main".to_string(),
///     values: Some(serde_json::json!({"cidr_block": "10.0.0.0/16"})),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformResource {
    /// Full terraform resource address including module path
    pub address: String,
    /// Resource mode (typically "managed" for regular resources)
    pub mode: String,
    /// Terraform resource type (e.g., "aws_vpc", "google_storage_bucket")
    pub r#type: String,
    /// Resource name within its module
    pub name: String,
    /// Optional configuration values and attributes for the resource
    pub values: Option<Value>,
}

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

/// Loads and parses a terraform provider schema from a JSON file
/// 
/// This function reads a terraform provider schema file and parses it into a
/// structured format suitable for analysis and processing. The schema contains
/// detailed information about all resource types, their attributes, constraints,
/// and metadata provided by terraform providers.
/// 
/// # Arguments
/// * `schema_path` - Path to the provider schema JSON file
/// 
/// # Returns
/// Nested HashMap structure organized as:
/// `provider_name -> resource_type -> attribute_name -> attribute_definition`
/// 
/// # Errors
/// - Failed to read the schema file
/// - Invalid JSON format in the schema file
/// - Missing expected structure in the schema
/// 
/// # Schema File Format
/// The expected schema file should be generated by `terraform providers schema -json`
/// and contain the standard terraform provider schema structure.
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::plan::load_provider_schema;
/// use std::path::Path;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let schema = load_provider_schema(Path::new("provider-schema.json"))?;
/// 
/// // Access schema for a specific resource type
/// if let Some(google_provider) = schema.get("google") {
///     if let Some(bucket_schema) = google_provider.get("google_storage_bucket") {
///         println!("Found {} attributes for google_storage_bucket", bucket_schema.len());
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn load_provider_schema(schema_path: &Path) -> Result<HashMap<String, HashMap<String, HashMap<String, Value>>>> {
    let schema_content = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;
    
    let schema_json: Value = serde_json::from_str(&schema_content)
        .with_context(|| "Failed to parse schema JSON")?;
    
    let mut result = HashMap::new();
    
    if let Some(provider_schemas) = schema_json.get("provider_schemas").and_then(|ps| ps.as_object()) {
        for (provider_name, provider_schema) in provider_schemas {
            let mut resource_map = HashMap::new();
            
            if let Some(resource_schemas) = provider_schema.get("resource_schemas").and_then(|rs| rs.as_object()) {
                for (resource_type, resource_schema) in resource_schemas {
                    let mut attribute_map = HashMap::new();
                    
                    if let Some(block) = resource_schema.get("block") {
                        if let Some(attributes) = block.get("attributes").and_then(|a| a.as_object()) {
                            for (attr_name, attr_def) in attributes {
                                attribute_map.insert(attr_name.clone(), attr_def.clone());
                            }
                        }
                    }
                    
                    resource_map.insert(resource_type.clone(), attribute_map);
                }
            }
            
            result.insert(provider_name.clone(), resource_map);
        }
    }
    
    Ok(result)
}

/// Scores resource attributes based on their likelihood of being useful ID fields
/// 
/// This function implements an intelligent scoring algorithm that analyzes terraform
/// resource attributes and assigns scores based on how likely each attribute is to
/// be useful as a resource identifier for import operations. Higher scores indicate
/// better ID candidates.
/// 
/// # Arguments
/// * `_resource_type` - Resource type being analyzed (currently unused but reserved for future enhancements)
/// * `attributes` - Map of attribute names to their schema definitions
/// 
/// # Returns
/// HashMap mapping attribute names to their calculated scores (0.0 to 100.0)
/// 
/// # Scoring Algorithm
/// The scoring algorithm prioritizes attributes based on these criteria:
/// - **"id" field**: 100.0 (highest priority - explicit ID field)
/// - **"name" field**: 90.0 (names are often unique identifiers)
/// - **Fields ending in "_id" or containing "self_link"**: 80.0 (likely ID fields)
/// - **Path or location fields**: 70.0 (often contain identifying information)
/// - **Required fields**: 60.0 (required fields may be identifying)
/// - **String type fields**: 50.0 (strings are often used for identifiers)
/// - **Computed fields**: 40.0 (computed values may be generated IDs)
/// - **Other fields**: 30.0 (baseline score for any other attribute)
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::plan::score_attributes_for_id;
/// use std::collections::HashMap;
/// use serde_json::json;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut attributes = HashMap::new();
/// attributes.insert("id".to_string(), json!({"type": "string", "computed": true}));
/// attributes.insert("name".to_string(), json!({"type": "string", "required": true}));
/// 
/// let scores = score_attributes_for_id("aws_vpc", &attributes)?;
/// 
/// // "id" should have the highest score
/// assert_eq!(scores.get("id"), Some(&100.0));
/// assert_eq!(scores.get("name"), Some(&90.0));
/// # Ok(())
/// # }
/// ```
pub fn score_attributes_for_id(_resource_type: &str, attributes: &HashMap<String, Value>) -> Result<HashMap<String, f64>> {
    let mut scores = HashMap::new();
    
    // Score each attribute
    for (name, def) in attributes {
        #[allow(unused_assignments)]
        let mut score = 0.0;
        
        // Explicit ID field gets highest score
        if name == "id" {
            score = 100.0;
        } 
        // Name fields are often good identifiers
        else if name == "name" {
            score = 90.0;
        }
        // Self links or resource-specific identifiers
        else if name.ends_with("_id") || name.contains("self_link") {
            score = 80.0;
        }
        // Path or location indicators
        else if name.contains("path") || name == "location" {
            score = 70.0;
        }
        // Unique constraints
        else if let Some(true) = def.get("computed").and_then(|v| v.as_bool()) {
            score = 40.0;
        }
        // Required fields might be identifying
        else if let Some(true) = def.get("required").and_then(|v| v.as_bool()) {
            score = 60.0;
        }
        // String types might be more useful as IDs than other types
        else if let Some("string") = def.get("type").and_then(|v| v.as_str()) {
            score = 50.0;
        }
        else {
            score = 30.0;
        }
        // Do not remove score variables
        scores.insert(name.clone(), score);
    }
    
    Ok(scores)
}

/// Extracts the top ID candidate fields for a resource type based on attribute scoring
/// 
/// This function combines the scoring algorithm with selection logic to identify
/// the most promising attribute names for use as resource identifiers. It scores
/// all attributes and returns the top candidates sorted by their scores.
/// 
/// # Arguments
/// * `resource_type` - Terraform resource type to analyze
/// * `schema` - Map of attribute names to their schema definitions
/// 
/// # Returns
/// Vector of attribute names sorted by score (highest first), limited to top 3 candidates
/// 
/// # Selection Logic
/// 1. Scores all attributes using the scoring algorithm
/// 2. Sorts attributes by score in descending order
/// 3. Returns the top 3 candidates to avoid overwhelming users with too many options
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::plan::get_id_candidate_fields;
/// use std::collections::HashMap;
/// use serde_json::json;
/// 
/// let mut schema = HashMap::new();
/// schema.insert("id".to_string(), json!({"type": "string", "computed": true}));
/// schema.insert("name".to_string(), json!({"type": "string", "required": true}));
/// schema.insert("arn".to_string(), json!({"type": "string", "computed": true}));
/// schema.insert("description".to_string(), json!({"type": "string", "optional": true}));
/// 
/// let candidates = get_id_candidate_fields("aws_vpc", &schema);
/// 
/// // Should return top candidates: ["id", "name", "arn"] (in order of score)
/// assert_eq!(candidates.len(), 3);
/// assert_eq!(candidates[0], "id");  // Highest score
/// assert_eq!(candidates[1], "name"); // Second highest
/// ```
pub fn get_id_candidate_fields(resource_type: &str, schema: &HashMap<String, Value>) -> Vec<String> {
    if let Ok(scores) = score_attributes_for_id(resource_type, schema) {
        let mut scored_attrs: Vec<(String, f64)> = scores.into_iter().collect();
        scored_attrs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top 3 candidates
        scored_attrs.iter()
            .take(3)
            .map(|(name, _)| name.clone())
            .collect()
    } else {
        vec![]
    }
}