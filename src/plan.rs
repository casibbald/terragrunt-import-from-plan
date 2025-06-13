use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformPlan {
    pub planned_values: PlannedValues,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedValues {
    pub root_module: RootModule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootModule {
    pub child_modules: Vec<ChildModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildModule {
    pub resources: Vec<TerraformResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformResource {
    pub address: String,
    pub mode: String,
    pub r#type: String,
    pub name: String,
    pub values: Option<Value>,
}


use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};




/// Load and parse a provider schema from a JSON file
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

/// Score attributes based on how likely they are to be ID fields
pub fn score_attributes_for_id(_resource_type: &str, attributes: &HashMap<String, Value>) -> Result<HashMap<String, f64>> {
    let mut scores = HashMap::new();
    
    // Score each attribute
    for (name, def) in attributes {
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

/// Extract the top candidate fields for ID based on scoring
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