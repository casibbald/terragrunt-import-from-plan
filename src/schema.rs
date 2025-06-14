//! # Provider Schema Management Module
//! 
//! This module handles the loading, parsing, and management of terraform provider schemas.
//! It provides functionality for generating provider schemas using terragrunt and managing
//! schema metadata for intelligent resource ID inference.
//! 
//! ## Key Components
//! 
//! - **Schema Generation**: Generate provider schemas using terragrunt commands
//! - **Schema Management**: Load and cache provider schema information
//! - **Attribute Metadata**: Parse and analyze resource attribute metadata
//! - **Error Handling**: Comprehensive error types for schema operations
//! 
//! ## Sub-modules
//! 
//! - `manager`: Core schema management functionality
//! - `metadata`: Attribute metadata parsing and analysis

pub mod manager;
pub mod metadata;

use std::fs;
use std::path::Path;
use std::process::Command;
use serde_json::Value;
use thiserror::Error;

pub use manager::SchemaManager;
pub use metadata::{AttributeMetadata, ResourceAttributeMap, AttributeMetadataError};

/// Error types for provider schema operations
/// 
/// Represents various failure modes when working with terraform provider schemas,
/// providing detailed context about what went wrong during schema operations.
#[derive(Error, Debug)]
pub enum SchemaError {
    /// Failed to execute terragrunt command
    #[error("Failed to run terragrunt command: {0}")]
    CommandError(#[from] std::io::Error),
    /// Terragrunt command executed but returned non-zero exit status
    #[error("Terragrunt command failed with status {status}: stdout={stdout}, stderr={stderr}")]
    TerragruntError { 
        /// Exit status code from terragrunt command
        status: i32, 
        /// Standard output from the command
        stdout: String, 
        /// Standard error output from the command
        stderr: String 
    },
    /// Failed to parse the generated schema JSON
    #[error("Failed to parse schema JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    /// Failed to write schema file to disk
    #[error("Failed to write schema file: {0}")]
    WriteError(String),
}

/// Generates and writes terraform provider schema to a JSON file
/// 
/// This function executes `terragrunt providers schema -json` in the specified directory
/// and writes the resulting provider schema to `.terragrunt-provider-schema.json`.
/// The schema contains detailed information about all resource types and their attributes
/// supported by the terraform providers in use.
/// 
/// # Arguments
/// * `dir` - Directory containing terragrunt configuration to generate schema for
/// 
/// # Returns
/// Result indicating success or failure with detailed error information
/// 
/// # Errors
/// - `CommandError`: Failed to execute terragrunt command
/// - `TerragruntError`: Terragrunt command failed with non-zero exit status
/// - `JsonError`: Generated schema is not valid JSON
/// - `WriteError`: Failed to write schema file to disk
/// 
/// # Generated File
/// Creates `.terragrunt-provider-schema.json` in the specified directory containing
/// the complete provider schema information in JSON format.
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::schema::write_provider_schema;
/// use std::path::Path;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// write_provider_schema(Path::new("./envs/dev"))?;
/// # Ok(())
/// # }
/// ```
pub fn write_provider_schema(dir: &Path) -> Result<(), SchemaError> {
    let output = Command::new("terragrunt")
        .arg("providers")
        .arg("schema")
        .arg("-json")
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        return Err(SchemaError::TerragruntError {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    // Parse the JSON to validate it
    let _schema_json: Value = serde_json::from_slice(&output.stdout)?;

    // Write the schema to the expected file
    let schema_path = dir.join(".terragrunt-provider-schema.json");
    fs::write(&schema_path, &output.stdout)
        .map_err(|e| SchemaError::WriteError(format!("Failed to write to {}: {}", schema_path.display(), e)))?;

    Ok(())
} 