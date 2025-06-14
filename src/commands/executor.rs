//! # Import Command Executor Module
//! 
//! This module provides functionality for executing terragrunt import commands, both
//! individually and in batches. It handles command execution, error reporting, timing,
//! and dry-run simulation.
//! 
//! ## Key Features
//! 
//! - **Command Execution**: Execute terragrunt import commands with proper error handling
//! - **Batch Processing**: Execute multiple commands sequentially (required for terraform state)
//! - **Dry-Run Support**: Simulate command execution without making changes
//! - **Performance Tracking**: Track execution times and batch statistics
//! - **Comprehensive Error Handling**: Detailed error reporting with exit codes and output
//! 
//! ## Important Notes
//! 
//! - **Sequential Execution Only**: Commands MUST be executed sequentially due to terraform
//!   state management limitations. Concurrent execution would cause state corruption.
//! - **Directory Validation**: Working directories are validated before command execution
//! - **Output Capture**: Both stdout and stderr are captured for error analysis
//! 
//! ## Usage Pattern
//! 
//! 1. Create ImportCommand objects using the builder module
//! 2. Use ImportExecutor to execute commands individually or in batches
//! 3. Handle results based on success/failure with detailed error information

use std::path::PathBuf;
use std::process::Command;
use anyhow::Result;
use thiserror::Error;

/// Represents a terragrunt import command ready to be executed
/// 
/// This structure contains all the information needed to execute a terragrunt import
/// command, including the working directory, resource details, and identifiers.
/// Commands are typically created by the ImportCommandBuilder and then executed
/// by the ImportExecutor.
/// 
/// # Fields
/// - `working_directory`: Directory where terragrunt should be executed
/// - `resource_address`: Full terraform resource address (e.g., "module.vpc.aws_vpc.main")
/// - `resource_id`: Cloud resource ID to import
/// - `resource_type`: Terraform resource type (e.g., "aws_vpc")
/// - `module_name`: Name of the terragrunt module
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::commands::executor::ImportCommand;
/// use std::path::PathBuf;
/// 
/// let command = ImportCommand {
///     working_directory: PathBuf::from("./modules/vpc"),
///     resource_address: "aws_vpc.main".to_string(),
///     resource_id: "vpc-12345".to_string(),
///     resource_type: "aws_vpc".to_string(),
///     module_name: "vpc".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ImportCommand {
    /// Directory where the terragrunt command should be executed
    pub working_directory: PathBuf,
    /// Full terraform resource address to import
    pub resource_address: String,
    /// Cloud resource ID to import into terraform state
    pub resource_id: String,
    /// Terraform resource type for categorization
    pub resource_type: String,
    /// Name of the terragrunt module for reference
    pub module_name: String,
}

/// Result of executing a single import command
/// 
/// This enum represents the possible outcomes when executing a terragrunt import
/// command. It provides detailed information about successful executions, failures
/// with comprehensive error details, and dry-run simulations.
/// 
/// # Variants
/// - `Success`: Command executed successfully with timing information
/// - `Failed`: Command failed with detailed error information
/// - `DryRun`: Dry-run simulation showing command without execution
#[derive(Debug)]
pub enum ImportResult {
    /// Command executed successfully
    Success {
        /// Resource address that was successfully imported
        address: String,
        /// Execution time in milliseconds
        execution_time_ms: u128,
    },
    /// Command failed with detailed error information
    Failed {
        /// Resource address that failed to import
        address: String,
        /// High-level error description
        error: String,
        /// Standard error output from terragrunt command
        stderr: String,
        /// Standard output from terragrunt command
        stdout: String,
        /// Exit code from the terragrunt process
        exit_code: i32,
    },
    /// Dry-run result showing command without execution
    DryRun {
        /// Resource address for the dry run
        address: String,
        /// Full command string that would be executed
        command_string: String,
    },
}

/// Result of executing a batch of import commands
/// 
/// This structure provides comprehensive statistics and results for batch operations,
/// including categorized results, timing information, and summary counts.
/// 
/// # Fields
/// - `successful`: Vector of successful import results
/// - `failed`: Vector of failed import results  
/// - `total_executed`: Total number of commands processed
/// - `total_duration_ms`: Total time taken for the entire batch
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::commands::executor::BatchResult;
/// 
/// fn process_batch_result(result: BatchResult) {
///     println!("Executed {} commands in {}ms", 
///              result.total_executed, result.total_duration_ms);
///     println!("Success: {}, Failed: {}", 
///              result.successful.len(), result.failed.len());
/// }
/// ```
#[derive(Debug)]
pub struct BatchResult {
    /// Vector of successful import results with execution details
    pub successful: Vec<ImportResult>,
    /// Vector of failed import results with error details
    pub failed: Vec<ImportResult>,
    /// Total number of commands that were processed
    pub total_executed: usize,
    /// Total duration for the entire batch operation in milliseconds
    pub total_duration_ms: u128,
}

/// Error types for import execution
/// 
/// This enum defines the various error conditions that can occur during
/// import command execution, providing detailed context for each failure mode.
/// 
/// # Variants
/// - `DirectoryNotFound`: Working directory doesn't exist
/// - `CommandFailed`: Failed to execute the terragrunt command
/// - `TerragruntFailed`: Terragrunt executed but returned non-zero exit code
#[derive(Error, Debug)]
pub enum ImportExecutionError {
    /// Working directory for the command does not exist
    #[error("Working directory does not exist: {path}")]
    DirectoryNotFound { 
        /// Path that was not found
        path: String 
    },
    /// Failed to execute the terragrunt command process
    #[error("Failed to execute terragrunt command: {source}")]
    CommandFailed { 
        /// Underlying I/O error from command execution
        source: std::io::Error 
    },
    /// Terragrunt command executed but failed with non-zero exit code
    #[error("Terragrunt import failed with exit code {exit_code}: {stderr}")]
    TerragruntFailed {
        /// Exit code returned by terragrunt
        exit_code: i32,
        /// Standard error output from terragrunt
        stderr: String,
        /// Standard output from terragrunt
        stdout: String,
    },
}

/// Executor for running terragrunt import commands
/// 
/// The ImportExecutor provides methods for executing terragrunt import commands
/// both individually and in batches. It handles all aspects of command execution
/// including directory validation, process management, output capture, timing,
/// and error handling.
/// 
/// # Key Features
/// - **Safe Execution**: Validates working directories before execution
/// - **Error Handling**: Comprehensive error capture and reporting
/// - **Performance Tracking**: Measures execution times for analysis
/// - **Batch Operations**: Sequential batch processing for multiple commands
/// - **Dry-Run Support**: Simulate executions without making changes
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::commands::executor::{ImportExecutor, ImportCommand};
/// 
/// let executor = ImportExecutor;
/// let result = executor.execute_command(&command)?;
/// match result {
///     ImportResult::Success { address, execution_time_ms } => {
///         println!("✅ Imported {} in {}ms", address, execution_time_ms);
///     }
///     ImportResult::Failed { address, error, .. } => {
///         println!("❌ Failed to import {}: {}", address, error);
///     }
///     _ => {}
/// }
/// ```
pub struct ImportExecutor;

impl ImportExecutor {
    /// Executes a single terragrunt import command
    /// 
    /// This method executes a terragrunt import command in the specified working
    /// directory, capturing output, measuring execution time, and providing detailed
    /// results or error information.
    /// 
    /// # Arguments
    /// * `command` - ImportCommand containing all execution details
    /// 
    /// # Returns
    /// Result containing ImportResult on success or ImportExecutionError on failure
    /// 
    /// # Process
    /// 1. Validates working directory exists
    /// 2. Executes `terragrunt import {resource_address} {resource_id}`
    /// 3. Captures stdout, stderr, and exit code
    /// 4. Measures execution time
    /// 5. Returns structured result based on success/failure
    /// 
    /// # Errors
    /// - `DirectoryNotFound`: Working directory doesn't exist
    /// - `CommandFailed`: Failed to start terragrunt process
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::commands::executor::ImportExecutor;
    /// 
    /// let executor = ImportExecutor;
    /// match executor.execute_command(&command)? {
    ///     ImportResult::Success { address, execution_time_ms } => {
    ///         println!("✅ Successfully imported {} in {}ms", address, execution_time_ms);
    ///     }
    ///     ImportResult::Failed { address, error, stderr, .. } => {
    ///         eprintln!("❌ Failed to import {}: {}", address, error);
    ///         eprintln!("Error details: {}", stderr);
    ///     }
    ///     _ => unreachable!(),
    /// }
    /// ```
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

    /// Executes a batch of import commands sequentially
    /// 
    /// This method processes multiple import commands one by one, collecting results
    /// and providing comprehensive batch statistics. Commands are executed sequentially
    /// to avoid terraform state conflicts.
    /// 
    /// # Arguments
    /// * `commands` - Slice of ImportCommand objects to execute
    /// 
    /// # Returns
    /// BatchResult containing categorized results and timing statistics
    /// 
    /// # Important Notes
    /// - **Sequential Execution**: Commands MUST be executed sequentially due to terraform
    ///   state management limitations. Concurrent execution would cause state corruption
    ///   and locking conflicts.
    /// - **Continues on Failure**: If one command fails, execution continues with remaining commands
    /// - **Comprehensive Tracking**: Tracks both successful and failed operations
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::commands::executor::ImportExecutor;
    /// 
    /// let executor = ImportExecutor;
    /// let result = executor.execute_batch(&commands);
    /// 
    /// println!("Batch Execution Results:");
    /// println!("  Total: {} commands in {}ms", 
    ///          result.total_executed, result.total_duration_ms);
    /// println!("  Successful: {}", result.successful.len());
    /// println!("  Failed: {}", result.failed.len());
    /// 
    /// for success in &result.successful {
    ///     if let ImportResult::Success { address, execution_time_ms } = success {
    ///         println!("  ✅ {} ({}ms)", address, execution_time_ms);
    ///     }
    /// }
    /// ```
    pub fn execute_batch(&self, commands: &[ImportCommand]) -> BatchResult {
        let start_time = std::time::Instant::now();
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for command in commands {
            match self.execute_command(command) {
                Ok(result @ ImportResult::Success { .. }) => {
                    successful.push(result);
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

    /// Creates a dry-run result without executing the command
    /// 
    /// This method simulates command execution by generating the exact command string
    /// that would be executed, without actually running terragrunt. Useful for
    /// previewing operations or debugging command generation.
    /// 
    /// # Arguments
    /// * `command` - ImportCommand to simulate
    /// 
    /// # Returns
    /// ImportResult::DryRun containing the command string and resource address
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::commands::executor::ImportExecutor;
    /// 
    /// let executor = ImportExecutor;
    /// let result = executor.dry_run_command(&command);
    /// 
    /// if let ImportResult::DryRun { address, command_string } = result {
    ///     println!("🌿 [DRY RUN] Would execute for {}:", address);
    ///     println!("    {}", command_string);
    /// }
    /// ```
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

    /// Creates dry-run results for a batch of commands
    /// 
    /// This method simulates batch execution by generating command strings for all
    /// commands without executing any of them. Useful for previewing large batches
    /// or validating command generation before actual execution.
    /// 
    /// # Arguments
    /// * `commands` - Slice of ImportCommand objects to simulate
    /// 
    /// # Returns
    /// Vector of ImportResult::DryRun results, one for each input command
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::commands::executor::ImportExecutor;
    /// 
    /// let executor = ImportExecutor;
    /// let dry_run_results = executor.dry_run_batch(&commands);
    /// 
    /// println!("🌿 Dry Run Results ({} commands):", dry_run_results.len());
    /// for result in dry_run_results {
    ///     if let ImportResult::DryRun { address, command_string } = result {
    ///         println!("  {} -> {}", address, command_string);
    ///     }
    /// }
    /// ```
    pub fn dry_run_batch(&self, commands: &[ImportCommand]) -> Vec<ImportResult> {
        commands
            .iter()
            .map(|command| self.dry_run_command(command))
            .collect()
    }
} 