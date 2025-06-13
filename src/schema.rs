use std::fs;
use std::path::Path;
use std::process::Command;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Failed to run terragrunt command: {0}")]
    CommandError(#[from] std::io::Error),
    #[error("Terragrunt command failed with status {status}: stdout={stdout}, stderr={stderr}")]
    TerragruntError { status: i32, stdout: String, stderr: String },
    #[error("Failed to parse schema JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to write schema file: {0}")]
    WriteError(String),
}

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