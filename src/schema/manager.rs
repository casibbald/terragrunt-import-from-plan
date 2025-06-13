use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use serde_json::Value;
use anyhow::{Result, Context};
use crate::importer::PlanFile;
use crate::schema::{write_provider_schema, AttributeMetadata, ResourceAttributeMap, AttributeMetadataError};

/// Centralized schema management for provider schemas and ID candidate extraction
#[derive(Debug)]
pub struct SchemaManager {
    cache: HashMap<String, Value>,
    working_dir: PathBuf,
}

impl SchemaManager {
    /// Create a new SchemaManager with the specified working directory
    pub fn new<P: AsRef<Path>>(working_dir: P) -> Self {
        Self {
            cache: HashMap::new(),
            working_dir: working_dir.as_ref().to_path_buf(),
        }
    }

    /// Load or generate the provider schema, caching the result
    pub fn load_or_generate_schema(&mut self) -> Result<&Value> {
        // Check if we have it cached
        if self.cache.contains_key("provider_schema") {
            return Ok(self.cache.get("provider_schema").unwrap());
        }

        // Try to load from file first
        let schema_path = self.working_dir.join(".terragrunt-provider-schema.json");
        
        let schema_value = if schema_path.exists() {
            let content = std::fs::read_to_string(&schema_path)
                .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;
            
            serde_json::from_str(&content)
                .with_context(|| "Failed to parse cached schema JSON")?
        } else {
            // Generate new schema
            write_provider_schema(&self.working_dir)
                .with_context(|| "Failed to generate provider schema")?;
            
            let content = std::fs::read_to_string(&schema_path)
                .with_context(|| format!("Failed to read generated schema file: {}", schema_path.display()))?;
            
            serde_json::from_str(&content)
                .with_context(|| "Failed to parse generated schema JSON")?
        };

        // Cache the result
        self.cache.insert("provider_schema".to_string(), schema_value);
        
        Ok(self.cache.get("provider_schema").unwrap())
    }

    /// Get the schema for a specific resource type from a provider
    pub fn get_resource_schema(&self, provider: &str, resource_type: &str) -> Option<&Value> {
        self.cache
            .get("provider_schema")?
            .get("provider_schemas")?
            .get(provider)?
            .get("resource_schemas")?
            .get(resource_type)
    }

    /// Extract ID candidate fields from the provider schema for a specific resource type
    pub fn extract_id_candidates(&self, resource_type: &str) -> HashSet<String> {
        let mut candidates = HashSet::new();

        // Check if we have cached schema
        if let Some(schema) = self.cache.get("provider_schema") {
            if let Some(resource_schemas) = schema
                .get("provider_schemas")
                .and_then(|ps| ps.get("google")) // TODO: Make provider configurable
                .and_then(|g| g.get("resource_schemas"))
                .and_then(|rs| rs.as_object())
            {
                if let Some(resource_schema) = resource_schemas.get(resource_type) {
                    if let Some(block) = resource_schema.get("block") {
                        if let Some(attributes) = block.get("attributes").and_then(|a| a.as_object()) {
                            for (attr_name, _) in attributes {
                                candidates.insert(attr_name.clone());
                            }
                        }
                    }
                }
            }
        }

        candidates
    }

    /// Extract schema map from a plan file (for backward compatibility)
    pub fn extract_schema_map_from_plan(plan: &PlanFile) -> HashMap<String, Value> {
        plan.provider_schemas
            .as_ref()
            .and_then(|ps| ps.provider_schemas.values().next())
            .and_then(|provider| provider.resource_schemas.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    /// Extract ID candidate fields from schema JSON (static method for backward compatibility)
    pub fn extract_id_candidate_fields_from_schema(schema_json: &Value) -> HashSet<String> {
        let mut candidates = HashSet::new();

        if let Some(resource_schemas) = schema_json
            .get("provider_schemas")
            .and_then(|ps| ps.get("google")) // assumes Google provider
            .and_then(|g| g.get("resource_schemas"))
            .and_then(|rs| rs.as_object())
        {
            for (_resource_type, schema) in resource_schemas {
                if let Some(block) = schema.get("block") {
                    if let Some(attributes) = block.get("attributes").and_then(|a| a.as_object()) {
                        for (attr_name, _) in attributes {
                            candidates.insert(attr_name.clone());
                        }
                    }
                }
            }
        }

        candidates
    }

    /// Fallback method to extract ID candidates from resource values when no schema is available
    pub fn extract_id_candidates_from_values(values: &serde_json::Map<String, Value>) -> HashSet<String> {
        let mut fields = HashSet::new();

        for (key, val) in values.iter() {
            if val.is_string() || val.is_number() || val.is_boolean() {
                fields.insert(key.clone());
            }
        }

        fields
    }

    /// Clear the schema cache (useful for testing or forced refresh)
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Check if schema is cached
    pub fn has_cached_schema(&self) -> bool {
        self.cache.contains_key("provider_schema")
    }

    /// Parse resource attributes with full metadata for a specific resource type
    /// 
    /// This extracts detailed attribute information from the cached schema,
    /// including whether attributes are required, computed, optional, their types, etc.
    pub fn parse_resource_attributes(&self, resource_type: &str) -> Result<ResourceAttributeMap, AttributeMetadataError> {
        let mut attributes = HashMap::new();
        
        // Get the cached schema
        let schema = self.cache.get("provider_schema")
            .ok_or_else(|| AttributeMetadataError::ParseError { 
                message: "No schema loaded. Call load_or_generate_schema() first.".to_string() 
            })?;

        // Navigate to the resource schema: 
        // .provider_schemas["registry.terraform.io/hashicorp/google"].resource_schemas[resource_type].block.attributes
        let resource_schema = schema
            .get("provider_schemas")
            .and_then(|ps| ps.as_object())
            .and_then(|ps_obj| {
                // Try different provider name patterns
                ps_obj.get("registry.terraform.io/hashicorp/google")
                    .or_else(|| ps_obj.get("google"))
                    .or_else(|| ps_obj.values().next()) // Fallback to first provider
            })
            .and_then(|provider| provider.get("resource_schemas"))
            .and_then(|rs| rs.get(resource_type))
            .and_then(|resource| resource.get("block"))
            .and_then(|block| block.get("attributes"))
            .and_then(|attrs| attrs.as_object())
            .ok_or_else(|| AttributeMetadataError::ParseError {
                message: format!("Could not find attributes for resource type: {}", resource_type)
            })?;

        // Parse each attribute into AttributeMetadata
        for (attr_name, attr_value) in resource_schema {
            match AttributeMetadata::from_schema_value(attr_value) {
                Ok(metadata) => {
                    attributes.insert(attr_name.clone(), metadata);
                }
                Err(e) => {
                    eprintln!("⚠️ Warning: Failed to parse attribute '{}' for resource '{}': {}", attr_name, resource_type, e);
                    // Continue processing other attributes instead of failing completely
                }
            }
        }

        Ok(attributes)
    }

    /// Get metadata for a specific attribute of a resource type
    pub fn get_attribute_metadata(&self, resource_type: &str, attr_name: &str) -> Result<Option<AttributeMetadata>, AttributeMetadataError> {
        let attributes = self.parse_resource_attributes(resource_type)?;
        Ok(attributes.get(attr_name).cloned())
    }

    /// Get all potential ID candidate attributes for a resource type using real schema metadata
    /// 
    /// This replaces the old hardcoded approach with schema-driven intelligence
    pub fn get_id_candidate_attributes(&self, resource_type: &str) -> Result<Vec<(String, AttributeMetadata)>, AttributeMetadataError> {
        let attributes = self.parse_resource_attributes(resource_type)?;
        
        let mut candidates: Vec<(String, AttributeMetadata)> = attributes
            .into_iter()
            .filter(|(_, metadata)| metadata.is_potential_id())
            .collect();
        
        // Sort by base score (highest first)
        candidates.sort_by(|a, b| {
            b.1.calculate_base_score()
                .partial_cmp(&a.1.calculate_base_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(candidates)
    }

    /// List all available resource types in the loaded schema
    pub fn list_resource_types(&self) -> Result<Vec<String>, AttributeMetadataError> {
        let schema = self.cache.get("provider_schema")
            .ok_or_else(|| AttributeMetadataError::ParseError { 
                message: "No schema loaded. Call load_or_generate_schema() first.".to_string() 
            })?;

        let resource_types = schema
            .get("provider_schemas")
            .and_then(|ps| ps.as_object())
            .and_then(|ps_obj| {
                // Try different provider name patterns
                ps_obj.get("registry.terraform.io/hashicorp/google")
                    .or_else(|| ps_obj.get("google"))
                    .or_else(|| ps_obj.values().next())
            })
            .and_then(|provider| provider.get("resource_schemas"))
            .and_then(|rs| rs.as_object())
            .map(|rs_obj| rs_obj.keys().cloned().collect())
            .ok_or_else(|| AttributeMetadataError::ParseError {
                message: "Could not extract resource types from schema".to_string()
            })?;

        Ok(resource_types)
    }
} 