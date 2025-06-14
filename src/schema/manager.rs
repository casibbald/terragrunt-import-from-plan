//! # Schema Manager Module
//! 
//! This module provides centralized management of terraform provider schemas, including
//! caching, loading, and intelligent analysis of resource attributes for ID inference.
//! 
//! ## Key Functionality
//! 
//! - **Schema Loading**: Load schemas from files or generate via terragrunt
//! - **Caching**: Efficient caching of parsed schema information
//! - **ID Inference**: Intelligent extraction of ID candidate fields
//! - **Metadata Analysis**: Rich attribute metadata parsing and analysis
//! - **Multi-Provider Support**: Handle schemas from multiple terraform providers

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use serde_json::Value;
use anyhow::{Result, Context};
use crate::importer::PlanFile;
use crate::schema::{write_provider_schema, AttributeMetadata, ResourceAttributeMap, AttributeMetadataError};

/// Centralized schema management for provider schemas and ID candidate extraction
/// 
/// The SchemaManager provides a high-level interface for working with terraform provider
/// schemas. It handles loading schemas from files or generating them via terragrunt,
/// caches parsed information for efficiency, and provides intelligent analysis of
/// resource attributes for ID inference.
/// 
/// # Key Features
/// - **Automatic Loading**: Load schemas from files or generate as needed
/// - **Efficient Caching**: Cache parsed schemas to avoid repeated parsing
/// - **Smart ID Inference**: Use schema metadata to identify potential ID fields
/// - **Provider Agnostic**: Work with schemas from any terraform provider
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = SchemaManager::new("./envs/dev");
/// let schema = manager.load_or_generate_schema()?;
/// let candidates = manager.extract_id_candidates("google_storage_bucket");
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SchemaManager {
    /// Cached schema data to avoid repeated file operations
    cache: HashMap<String, Value>,
    /// Working directory for terragrunt operations and schema files
    working_dir: PathBuf,
}

impl SchemaManager {
    /// Creates a new SchemaManager with the specified working directory
    /// 
    /// The working directory should contain terragrunt configuration and will be
    /// used for generating provider schemas when needed. The manager will look
    /// for `.terragrunt-provider-schema.json` in this directory.
    /// 
    /// # Arguments
    /// * `working_dir` - Directory containing terragrunt configuration
    /// 
    /// # Returns
    /// A new SchemaManager instance ready for schema operations
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::schema::SchemaManager;
    /// 
    /// let manager = SchemaManager::new("./envs/dev");
    /// let manager2 = SchemaManager::new("/path/to/terragrunt/config");
    /// ```
    pub fn new<P: AsRef<Path>>(working_dir: P) -> Self {
        Self {
            cache: HashMap::new(),
            working_dir: working_dir.as_ref().to_path_buf(),
        }
    }

    /// Loads or generates the provider schema, caching the result for efficiency
    /// 
    /// This method first checks if a schema is already cached in memory. If not,
    /// it attempts to load from `.terragrunt-provider-schema.json` in the working
    /// directory. If that file doesn't exist, it generates a new schema using
    /// terragrunt and caches the result.
    /// 
    /// # Returns
    /// Reference to the loaded schema JSON
    /// 
    /// # Errors
    /// - Failed to read existing schema file
    /// - Failed to parse schema JSON
    /// - Failed to generate new schema via terragrunt
    /// 
    /// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = SchemaManager::new("./envs/dev");
/// let schema = manager.load_or_generate_schema()?;
/// println!("Loaded schema with {} providers", 
///          schema.get("provider_schemas").unwrap().as_object().unwrap().len());
/// # Ok(())
/// # }
/// ```
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

    /// Gets the schema definition for a specific resource type from a provider
    /// 
    /// This method retrieves the complete schema definition for a specific resource
    /// type, which includes information about all attributes, their types, and
    /// constraints. Useful for detailed schema analysis.
    /// 
    /// # Arguments
    /// * `provider` - Provider name (e.g., "google", "aws")
    /// * `resource_type` - Resource type (e.g., "google_storage_bucket")
    /// 
    /// # Returns
    /// Optional reference to the resource schema JSON, None if not found
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::schema::SchemaManager;
    /// 
    /// let manager = SchemaManager::new("./envs/dev");
    /// if let Some(schema) = manager.get_resource_schema("google", "google_storage_bucket") {
    ///     println!("Found schema for google_storage_bucket");
    /// }
    /// ```
    pub fn get_resource_schema(&self, provider: &str, resource_type: &str) -> Option<&Value> {
        self.cache
            .get("provider_schema")?
            .get("provider_schemas")?
            .get(provider)?
            .get("resource_schemas")?
            .get(resource_type)
    }

    /// Extracts ID candidate field names for a specific resource type
    /// 
    /// This method analyzes the provider schema for a specific resource type and
    /// extracts all attribute names that could potentially be used as resource IDs.
    /// This provides the foundation for intelligent ID inference.
    /// 
    /// # Arguments
    /// * `resource_type` - Resource type to analyze (e.g., "google_storage_bucket")
    /// 
    /// # Returns
    /// Set of attribute names that could be used as resource IDs
    /// 
    /// # Note
    /// Currently hardcoded to use Google provider. This will be made configurable
    /// in future versions.
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::schema::SchemaManager;
    /// 
    /// let manager = SchemaManager::new("./envs/dev");
    /// let candidates = manager.extract_id_candidates("google_storage_bucket");
    /// println!("Found {} potential ID fields", candidates.len());
    /// ```
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

    /// Extracts schema map from a plan file for backward compatibility
    /// 
    /// This static method provides backward compatibility with older code that
    /// extracted schema information directly from plan files. It extracts the
    /// provider schema information from a terraform plan.
    /// 
    /// # Arguments
    /// * `plan` - Terraform plan file containing provider schemas
    /// 
    /// # Returns
    /// HashMap mapping resource types to their schema definitions
    /// 
    /// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// use terragrunt_import_from_plan::importer::PlanFile;
/// use serde_json::json;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let plan = PlanFile {
///     format_version: "1.0".to_string(),
///     terraform_version: "1.0".to_string(),
///     variables: None,
///     planned_values: None,
///     provider_schemas: None,
/// };
/// let schema_map = SchemaManager::extract_schema_map_from_plan(&plan);
/// println!("Extracted {} resource schemas", schema_map.len());
/// # Ok(())
/// # }
/// ```
    pub fn extract_schema_map_from_plan(plan: &PlanFile) -> HashMap<String, Value> {
        plan.provider_schemas
            .as_ref()
            .and_then(|ps| ps.provider_schemas.values().next())
            .and_then(|provider| provider.resource_schemas.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    /// Extracts ID candidate fields from schema JSON (static method for backward compatibility)
    /// 
    /// This static method provides backward compatibility with older code that
    /// processed schema JSON directly. It extracts potential ID candidate fields
    /// from a complete provider schema.
    /// 
    /// # Arguments
    /// * `schema_json` - Complete provider schema in JSON format
    /// 
    /// # Returns
    /// Set of all attribute names found across all resource types
    /// 
    /// # Note
    /// Currently assumes Google provider. This is a legacy method maintained
    /// for backward compatibility.
    /// 
    /// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// use serde_json::json;
/// 
/// let schema_json = json!({
///     "provider_schemas": {
///         "google": {
///             "resource_schemas": {
///                 "google_storage_bucket": {
///                     "block": {
///                         "attributes": {
///                             "name": {},
///                             "location": {}
///                         }
///                     }
///                 }
///             }
///         }
///     }
/// });
/// let candidates = SchemaManager::extract_id_candidate_fields_from_schema(&schema_json);
/// println!("Found {} total candidate fields", candidates.len());
/// ```
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
    /// 
    /// When provider schema information is not available, this method provides a
    /// basic fallback by examining the actual resource values and identifying
    /// attributes that have simple types (string, number, boolean) that could
    /// potentially be used as IDs.
    /// 
    /// # Arguments
    /// * `values` - Map of resource attribute values from terraform plan
    /// 
    /// # Returns
    /// Set of attribute names with simple types that could be ID candidates
    /// 
    /// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// use serde_json::{Map, Value, json};
/// 
/// let mut resource_values = Map::new();
/// resource_values.insert("name".to_string(), json!("my-bucket"));
/// resource_values.insert("location".to_string(), json!("us-central1"));
/// resource_values.insert("description".to_string(), json!("Test bucket"));
/// 
/// let candidates = SchemaManager::extract_id_candidates_from_values(&resource_values);
/// println!("Found {} potential ID fields from values", candidates.len());
/// ```
    pub fn extract_id_candidates_from_values(values: &serde_json::Map<String, Value>) -> HashSet<String> {
        let mut fields = HashSet::new();

        for (key, val) in values.iter() {
            if val.is_string() || val.is_number() || val.is_boolean() {
                fields.insert(key.clone());
            }
        }

        fields
    }

    /// Clears the schema cache (useful for testing or forced refresh)
    /// 
    /// This method removes all cached schema information, forcing the next
    /// schema operation to reload from files or regenerate. Useful for testing
    /// or when you need to ensure fresh schema data.
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::schema::SchemaManager;
    /// 
    /// let mut manager = SchemaManager::new("./envs/dev");
    /// manager.clear_cache(); // Force reload on next schema access
    /// ```
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Checks if schema is currently cached in memory
    /// 
    /// This method allows you to determine if schema information is already
    /// loaded and cached, which can be useful for performance monitoring or
    /// debugging cache behavior.
    /// 
    /// # Returns
    /// True if schema is cached, false if it would need to be loaded
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::schema::SchemaManager;
    /// 
    /// let manager = SchemaManager::new("./envs/dev");
    /// if !manager.has_cached_schema() {
    ///     println!("Schema will need to be loaded");
    /// }
    /// ```
    pub fn has_cached_schema(&self) -> bool {
        self.cache.contains_key("provider_schema")
    }

    /// Parses resource attributes with full metadata for a specific resource type
    /// 
    /// This method extracts detailed attribute information from the cached schema,
    /// including whether attributes are required, computed, optional, their types,
    /// and other metadata useful for intelligent ID inference and validation.
    /// 
    /// # Arguments
    /// * `resource_type` - Resource type to analyze (e.g., "google_storage_bucket", "aws_s3_bucket")
    /// 
    /// # Returns
    /// Map of attribute names to their detailed metadata
    /// 
    /// # Errors
    /// - No schema loaded (call load_or_generate_schema() first)
    /// - Resource type not found in schema
    /// - Schema parsing errors
    /// 
    /// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = SchemaManager::new("./envs/dev");
/// manager.load_or_generate_schema()?;
/// let attributes = manager.parse_resource_attributes("google_storage_bucket")?;
/// println!("Found {} attributes", attributes.len());
/// # Ok(())
/// # }
/// ```
    pub fn parse_resource_attributes(&self, resource_type: &str) -> Result<ResourceAttributeMap, AttributeMetadataError> {
        let mut attributes = HashMap::new();
        
        // Get the cached schema
        let schema = self.cache.get("provider_schema")
            .ok_or_else(|| AttributeMetadataError::ParseError { 
                message: "No schema loaded. Call load_or_generate_schema() first.".to_string() 
            })?;

        // Auto-detect provider from resource type
        let provider_name = if resource_type.starts_with("google_") {
            "registry.terraform.io/hashicorp/google"
        } else if resource_type.starts_with("aws_") {
            "registry.terraform.io/hashicorp/aws"
        } else if resource_type.starts_with("azurerm_") {
            "registry.terraform.io/hashicorp/azurerm"
        } else if resource_type.starts_with("random_") {
            "registry.terraform.io/hashicorp/random"
        } else {
            // Try to detect from loaded providers
            return self.find_resource_in_any_provider(resource_type);
        };

        // Navigate to the resource schema: 
        // .provider_schemas[provider_name].resource_schemas[resource_type].block.attributes
        let resource_schema = schema
            .get("provider_schemas")
            .and_then(|ps| ps.as_object())
            .and_then(|ps_obj| ps_obj.get(provider_name))
            .and_then(|provider| provider.get("resource_schemas"))
            .and_then(|rs| rs.get(resource_type))
            .and_then(|resource| resource.get("block"))
            .and_then(|block| block.get("attributes"))
            .and_then(|attrs| attrs.as_object())
            .ok_or_else(|| AttributeMetadataError::ParseError {
                message: format!("Could not find attributes for resource type: {} in provider: {}", resource_type, provider_name)
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

    /// Gets metadata for a specific attribute of a resource type
    /// 
    /// This method retrieves detailed metadata for a single attribute, which can
    /// be useful when you need to analyze specific attributes rather than all
    /// attributes of a resource type.
    /// 
    /// # Arguments
    /// * `resource_type` - Resource type containing the attribute
    /// * `attr_name` - Name of the specific attribute to analyze
    /// 
    /// # Returns
    /// Optional AttributeMetadata for the specified attribute
    /// 
    /// # Errors
    /// - No schema loaded
    /// - Resource type not found
    /// - Schema parsing errors
    /// 
    /// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = SchemaManager::new("./envs/dev");
/// manager.load_or_generate_schema()?;
/// if let Some(metadata) = manager.get_attribute_metadata("google_storage_bucket", "name")? {
///     println!("Attribute 'name' score: {}", metadata.calculate_base_score());
/// }
/// # Ok(())
/// # }
/// ```
    pub fn get_attribute_metadata(&self, resource_type: &str, attr_name: &str) -> Result<Option<AttributeMetadata>, AttributeMetadataError> {
        let attributes = self.parse_resource_attributes(resource_type)?;
        Ok(attributes.get(attr_name).cloned())
    }

    /// Gets all potential ID candidate attributes for a resource type using real schema metadata
    /// 
    /// This method replaces older hardcoded approaches with schema-driven intelligence.
    /// It analyzes the schema metadata to identify attributes that are likely to be
    /// useful as resource IDs, returning them sorted by their calculated scores.
    /// 
    /// # Arguments
    /// * `resource_type` - Resource type to analyze for ID candidates
    /// 
    /// # Returns
    /// Vector of (attribute_name, metadata) tuples sorted by score (highest first)
    /// 
    /// # Errors
    /// - No schema loaded
    /// - Resource type not found
    /// - Schema parsing errors
    /// 
    /// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = SchemaManager::new("./envs/dev");
/// manager.load_or_generate_schema()?;
/// let candidates = manager.get_id_candidate_attributes("google_storage_bucket")?;
/// for (name, metadata) in candidates.iter().take(3) {
///     println!("Candidate: {} (score: {})", name, metadata.calculate_base_score());
/// }
/// # Ok(())
/// # }
/// ```
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

    /// Lists all available resource types in the loaded schema across all providers
    /// 
    /// This method extracts all resource type names from all loaded provider schemas,
    /// which can be useful for discovery, validation, or building resource type
    /// selection interfaces.
    /// 
    /// # Returns
    /// Vector of all resource type names found across all providers in the schema
    /// 
    /// # Errors
    /// - No schema loaded (call load_or_generate_schema() first)
    /// - Failed to extract resource types from schema
    /// 
    /// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::SchemaManager;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut manager = SchemaManager::new("./envs/dev");
/// manager.load_or_generate_schema()?;
/// let resource_types = manager.list_resource_types()?;
/// println!("Available resource types: {}", resource_types.len());
/// for resource_type in resource_types.iter().take(5) {
///     println!("  - {}", resource_type);
/// }
/// # Ok(())
/// # }
/// ```
    pub fn list_resource_types(&self) -> Result<Vec<String>, AttributeMetadataError> {
        let schema = self.cache.get("provider_schema")
            .ok_or_else(|| AttributeMetadataError::ParseError { 
                message: "No schema loaded. Call load_or_generate_schema() first.".to_string() 
            })?;

        let mut all_resource_types = Vec::new();

        // Collect resource types from all providers
        if let Some(provider_schemas) = schema.get("provider_schemas").and_then(|ps| ps.as_object()) {
            for (_provider_name, provider_data) in provider_schemas {
                if let Some(resource_schemas) = provider_data.get("resource_schemas").and_then(|rs| rs.as_object()) {
                    for resource_type in resource_schemas.keys() {
                        all_resource_types.push(resource_type.clone());
                    }
                }
            }
        }

        if all_resource_types.is_empty() {
            return Err(AttributeMetadataError::ParseError {
                message: "No resource types found in any provider schema".to_string()
            });
        }

        // Sort for consistent output
        all_resource_types.sort();
        
        Ok(all_resource_types)
    }

    /// Helper method to find a resource type in any available provider
    /// 
    /// This method searches through all loaded providers to find the specified
    /// resource type when it can't be detected from the resource name prefix.
    /// 
    /// # Arguments
    /// * `resource_type` - Resource type to search for
    /// 
    /// # Returns
    /// Map of attribute names to their detailed metadata
    /// 
    /// # Errors
    /// - Resource type not found in any provider
    /// - Schema parsing errors
    fn find_resource_in_any_provider(&self, resource_type: &str) -> Result<ResourceAttributeMap, AttributeMetadataError> {
        let mut attributes = HashMap::new();
        
        // Get the cached schema
        let schema = self.cache.get("provider_schema")
            .ok_or_else(|| AttributeMetadataError::ParseError { 
                message: "No schema loaded. Call load_or_generate_schema() first.".to_string() 
            })?;

        // Search through all providers for this resource type
        if let Some(provider_schemas) = schema.get("provider_schemas").and_then(|ps| ps.as_object()) {
            for (provider_name, provider_data) in provider_schemas {
                if let Some(resource_schema) = provider_data
                    .get("resource_schemas")
                    .and_then(|rs| rs.get(resource_type))
                    .and_then(|resource| resource.get("block"))
                    .and_then(|block| block.get("attributes"))
                    .and_then(|attrs| attrs.as_object())
                {
                    // Found the resource in this provider
                    for (attr_name, attr_value) in resource_schema {
                        match AttributeMetadata::from_schema_value(attr_value) {
                            Ok(metadata) => {
                                attributes.insert(attr_name.clone(), metadata);
                            }
                            Err(e) => {
                                eprintln!("⚠️ Warning: Failed to parse attribute '{}' for resource '{}' in provider '{}': {}", 
                                    attr_name, resource_type, provider_name, e);
                            }
                        }
                    }
                    return Ok(attributes);
                }
            }
        }

        Err(AttributeMetadataError::ParseError {
            message: format!("Resource type '{}' not found in any loaded provider", resource_type)
        })
    }
} 