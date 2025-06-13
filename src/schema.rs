use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Failed to run terragrunt: status={status}, stdout={stdout}, stderr={stderr}")]
    TerragruntError {
        status: i32,
        stdout: String,
        stderr: String,
    },
    #[error("Failed to write schema file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub fn write_provider_schema(dir: &Path) -> Result<(), SchemaError> {
    let output = Command::new("terragrunt")
        .arg("providers")
        .arg("schema")
        .arg("-json")
        .current_dir(dir)
        .output()
        .map_err(|e| SchemaError::TerragruntError {
            status: -1,
            stdout: String::new(),
            stderr: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(SchemaError::TerragruntError {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let schema_path = dir.join(".terragrunt-provider-schema.json");
    std::fs::write(schema_path, output.stdout)?;
    Ok(())
} 