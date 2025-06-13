use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, Context};
use thiserror::Error;

/// Represents a terragrunt import command ready to be executed
#[derive(Debug, Clone)]
pub struct ImportCommand {
    pub working_directory: PathBuf,
    pub resource_address: String,
    pub resource_id: String,
    pub resource_type: String,
    pub module_name: String,
}

/// Result of executing a single import command
#[derive(Debug)]
pub enum ImportResult {
    Success {
        address: String,
        execution_time_ms: u128,
    },
    Failed {
        address: String,
        error: String,
        stderr: String,
        stdout: String,
        exit_code: i32,
    },
    DryRun {
        address: String,
        command_string: String,
    },
}

/// Result of executing a batch of import commands
#[derive(Debug)]
pub struct BatchResult {
    pub successful: Vec<ImportResult>,
    pub failed: Vec<ImportResult>,
    pub total_executed: usize,
    pub total_duration_ms: u128,
}

/// Error types for import execution
#[derive(Error, Debug)]
pub enum ImportExecutionError {
    #[error("Working directory does not exist: {path}")]
    DirectoryNotFound { path: String },
    #[error("Failed to execute terragrunt command: {source}")]
    CommandFailed { source: std::io::Error },
    #[error("Terragrunt import failed with exit code {exit_code}: {stderr}")]
    TerragruntFailed {
        exit_code: i32,
        stderr: String,
        stdout: String,
    },
}

/// Executor for running terragrunt import commands
pub struct ImportExecutor;

impl ImportExecutor {
    /// Execute a single import command
    pub fn execute_command(&self, command: &ImportCommand) -> Result<ImportResult, ImportExecutionError> {
        if !command.working_directory.exists() {
            return Err(ImportExecutionError::DirectoryNotFound {
                path: command.working_directory.display().to_string(),
            });
        }

        let start_time = std::time::Instant::now();

        let output = Command::new("terragrunt")
            .arg("import")
            .arg(&command.resource_address)
            .arg(&command.resource_id)
            .current_dir(&command.working_directory)
            .output()
            .map_err(|source| ImportExecutionError::CommandFailed { source })?;

        let execution_time_ms = start_time.elapsed().as_millis();

        if output.status.success() {
            Ok(ImportResult::Success {
                address: command.resource_address.clone(),
                execution_time_ms,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            Ok(ImportResult::Failed {
                address: command.resource_address.clone(),
                error: format!("Import failed with exit code {}", exit_code),
                stderr,
                stdout,
                exit_code,
            })
        }
    }

    /// Execute a batch of import commands sequentially
    pub fn execute_batch(&self, commands: &[ImportCommand]) -> BatchResult {
        let start_time = std::time::Instant::now();
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for command in commands {
            match self.execute_command(command) {
                Ok(ImportResult::Success { .. }) => {
                    successful.push(self.execute_command(command).unwrap());
                }
                Ok(result @ ImportResult::Failed { .. }) => {
                    failed.push(result);
                }
                Err(err) => {
                    failed.push(ImportResult::Failed {
                        address: command.resource_address.clone(),
                        error: err.to_string(),
                        stderr: String::new(),
                        stdout: String::new(),
                        exit_code: -1,
                    });
                }
                _ => unreachable!(),
            }
        }

        let total_duration_ms = start_time.elapsed().as_millis();

        BatchResult {
            total_executed: commands.len(),
            successful,
            failed,
            total_duration_ms,
        }
    }

    /// Create a dry-run result without executing the command
    pub fn dry_run_command(&self, command: &ImportCommand) -> ImportResult {
        let command_string = format!(
            "terragrunt import -config-dir={} {} {}",
            command.working_directory.display(),
            command.resource_address,
            command.resource_id
        );

        ImportResult::DryRun {
            address: command.resource_address.clone(),
            command_string,
        }
    }

    /// Create dry-run results for a batch of commands
    pub fn dry_run_batch(&self, commands: &[ImportCommand]) -> Vec<ImportResult> {
        commands
            .iter()
            .map(|command| self.dry_run_command(command))
            .collect()
    }
} 