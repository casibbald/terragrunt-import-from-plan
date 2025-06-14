//! # Terragrunt Import from Plan - Main Application
//! 
//! This is the main entry point for the terragrunt import tool. It provides both a CLI interface
//! for various terragrunt operations and a legacy mode for processing Terraform plan files to
//! generate import commands.
//! 
//! ## Features
//! 
//! - **Import Mode**: Process Terraform plan files and generate/execute terragrunt import commands
//! - **Fixture Generation**: Create test fixtures for AWS, GCP, and Azure providers
//! - **Workspace Management**: Clean and manage terragrunt workspace files
//! - **Validation**: Validate Terraform formatting and configuration
//! - **Module Operations**: Initialize, plan, apply, and destroy terragrunt modules
//! 
//! ## Usage
//! 
//! The tool can be used in two modes:
//! 
//! 1. **Command Mode** (Recommended): Use specific subcommands for different operations
//! 2. **Legacy Mode**: Provide --plan and --modules arguments for import functionality

#![feature(let_chains)]

mod app;
mod commands;
mod errors;
mod importer;
mod plan;
mod reporting;
mod schema;
mod scoring;
mod utils;

use crate::app::load_input_files;
use crate::importer::{execute_or_print_imports, map_resources_to_modules};
use crate::utils::{run_terragrunt_init, write_provider_schema, generate_fixtures, clean_workspace, extract_id_candidate_fields, validate_terraform_format, validate_terraform_config, format_terraform_files, init_terragrunt, plan_terragrunt, apply_terragrunt, destroy_terragrunt};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

/// Main CLI structure for the terragrunt import tool
/// 
/// This structure defines both the new command-based interface and legacy arguments
/// for backward compatibility. When no subcommand is provided, the tool falls back
/// to legacy mode using the --plan and --modules arguments.
#[derive(Parser, Debug)]
#[command(name = "terragrunt_import_from_plan")]
#[command(about = "Terragrunt import and fixture generation tool", long_about = None)]
struct Args {
    /// Subcommand to execute (if not provided, uses legacy mode)
    #[command(subcommand)]
    command: Option<Commands>,

    // Legacy arguments for backwards compatibility  
    /// Path to Terraform plan JSON file (legacy mode)
    #[arg(long)]
    plan: Option<String>,

    /// Path to modules.json file (legacy mode)
    #[arg(long)]
    modules: Option<String>,

    /// Root directory for module resolution (legacy mode)
    #[arg(long)]
    module_root: Option<String>,

    /// Run in dry-run mode (show commands without executing) (legacy mode)
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Enable verbose output (legacy mode)
    #[arg(long, default_value_t = false)]
    verbose: bool,

    /// Working directory for operations (legacy mode)
    #[arg(long)]
    working_directory: Option<String>,
}

/// Available subcommands for the terragrunt import tool
/// 
/// Each command provides specific functionality for managing terragrunt workspaces,
/// generating fixtures, or performing terraform operations.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate fixture files for a provider (replaces just gen)
    GenerateFixtures {
        /// Provider to generate fixtures for (aws, gcp, azure)
        #[arg(value_parser = ["aws", "gcp", "azure"])]
        provider: String,
    },
    /// Clean workspace files (replaces just clean)
    Clean {
        /// Specific provider to clean (optional, cleans all if not specified)
        #[arg(value_parser = ["aws", "gcp", "azure"])]
        provider: Option<String>,
        /// Also remove test fixtures (deep clean)
        #[arg(long)]
        deep: bool,
    },
    /// Extract ID candidate fields from schema
    ExtractIdFields {
        /// Path to schema JSON file
        schema_file: String,
    },
    /// Validate terraform formatting and configuration (replaces just validate)
    Validate {
        /// Provider to validate (aws, gcp, azure)
        #[arg(value_parser = ["aws", "gcp", "azure"])]
        provider: String,
        /// Only check terraform formatting
        #[arg(long)]
        format_only: bool,
        /// Only run terraform validate
        #[arg(long)]
        terraform_only: bool,
    },
    /// Format terraform files (replaces just fmt)
    Fmt {
        /// Provider to format (aws, gcp, azure)
        #[arg(value_parser = ["aws", "gcp", "azure"])]
        provider: String,
        /// Check formatting without making changes
        #[arg(long)]
        check: bool,
    },
    /// Initialize terragrunt modules (replaces just init)
    Init {
        /// Provider to initialize (aws, gcp, azure)
        #[arg(value_parser = ["aws", "gcp", "azure"])]
        provider: String,
        /// Environment (default: dev)
        #[arg(long, default_value = "dev")]
        env: String,
        /// Continue on failure (safe mode)
        #[arg(long)]
        safe: bool,
    },
    /// Plan terragrunt modules (replaces just plan)
    Plan {
        /// Provider to plan (aws, gcp, azure)
        #[arg(value_parser = ["aws", "gcp", "azure"])]
        provider: String,
        /// Environment (default: dev)
        #[arg(long, default_value = "dev")]
        env: String,
        /// Environment variables (KEY=value format)
        #[arg(long)]
        vars: Option<String>,
        /// Continue on failure (safe mode)
        #[arg(long)]
        safe: bool,
    },
    /// Apply terragrunt modules (replaces just apply)
    Apply {
        /// Provider to apply (aws, gcp, azure)
        #[arg(value_parser = ["aws", "gcp", "azure"])]
        provider: String,
        /// Environment (default: dev)
        #[arg(long, default_value = "dev")]
        env: String,
        /// Skip confirmation prompt (auto-approve)
        #[arg(long)]
        auto_approve: bool,
        /// Continue on failure (safe mode)
        #[arg(long)]
        safe: bool,
    },
    /// Destroy terragrunt modules (replaces just destroy)
    Destroy {
        /// Provider to destroy (aws, gcp, azure)
        #[arg(value_parser = ["aws", "gcp", "azure"])]
        provider: String,
        /// Environment (default: dev)
        #[arg(long, default_value = "dev")]
        env: String,
        /// Skip confirmation prompt (auto-approve)
        #[arg(long)]
        auto_approve: bool,
        /// Continue on failure (safe mode)
        #[arg(long)]
        safe: bool,
    },
}

/// Sets up provider schema for import operations (legacy mode helper)
/// 
/// This function initializes the terragrunt environment and generates provider schema
/// information needed for the import process. It's designed to be fault-tolerant and
/// will continue execution even if some steps fail, logging warnings appropriately.
/// 
/// # Arguments
/// * `working_directory` - Optional working directory, defaults to current directory
/// 
/// # Returns
/// Always returns Ok(()), even if individual steps fail (with warnings printed)
/// 
/// # Note
/// This function is used only in legacy mode for backward compatibility
fn setup_provider_schema(working_directory: Option<&str>) -> Result<()> {
    let working_dir = working_directory.unwrap_or(".");
    
    if let Err(e) = run_terragrunt_init(working_dir) {
        eprintln!("⚠️ Warning: terragrunt init failed: {:#}", e);
        // Continue execution despite the error
    }

    if let Err(e) = write_provider_schema(Path::new(working_dir)) {
        eprintln!("⚠️ Failed to generate provider schema: {}", e);
    }
    
    Ok(())
}

/// Main application entry point
/// 
/// Parses command line arguments and dispatches to the appropriate handler function.
/// Supports both modern command-based interface and legacy argument-based interface
/// for backward compatibility.
/// 
/// # Command Mode
/// Uses subcommands like `generate-fixtures`, `clean`, `validate`, etc. for specific operations.
/// 
/// # Legacy Mode
/// When no subcommand is provided, requires `--plan` and `--modules` arguments to process
/// Terraform plan files and generate/execute import commands.
/// 
/// # Returns
/// Result indicating success or failure of the operation
/// 
/// # Errors
/// Returns errors for invalid arguments, missing files, or operation failures
fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Commands::GenerateFixtures { provider }) => {
            println!("🔧 Generating fixtures for {} provider...", provider);
            generate_fixtures(&provider)
        }
        Some(Commands::Clean { provider, deep }) => {
            println!("🧹 Cleaning workspace...");
            clean_workspace(provider.as_deref(), deep)
        }
        Some(Commands::ExtractIdFields { schema_file }) => {
            let schema_content = std::fs::read_to_string(&schema_file)
                .with_context(|| format!("Failed to read schema file: {}", schema_file))?;
            let schema_json: serde_json::Value = serde_json::from_str(&schema_content)
                .with_context(|| "Failed to parse schema JSON")?;
            let candidates = extract_id_candidate_fields(&schema_json);
            println!("ID candidate fields: {:?}", candidates);
            Ok(())
        }
        Some(Commands::Validate { provider, format_only, terraform_only }) => {
            println!("🔍 Running comprehensive validation for {}...", provider);
            
            if !terraform_only {
                validate_terraform_format(&provider)?;
            }
            
            if !format_only {
                validate_terraform_config(&provider)?;
            }
            
            println!("✅ Validation completed successfully for {}", provider);
            Ok(())
        }
        Some(Commands::Fmt { provider, check }) => {
            format_terraform_files(&provider, check)
        }
        Some(Commands::Init { provider, env, safe }) => {
            init_terragrunt(&provider, &env, safe)
        }
        Some(Commands::Plan { provider, env, vars, safe }) => {
            plan_terragrunt(&provider, &env, vars.as_deref(), safe)
        }
        Some(Commands::Apply { provider, env, auto_approve, safe }) => {
            apply_terragrunt(&provider, &env, auto_approve, safe)
        }
        Some(Commands::Destroy { provider, env, auto_approve, safe }) => {
            destroy_terragrunt(&provider, &env, auto_approve, safe)
        }
        None => {
            // Legacy mode - require plan and modules arguments
            let plan = args.plan.ok_or_else(|| anyhow::anyhow!("--plan argument is required when not using subcommands"))?;
            let modules = args.modules.ok_or_else(|| anyhow::anyhow!("--modules argument is required when not using subcommands"))?;
            
            let (modules_file, plan_file) = load_input_files(&modules, &plan)
                .context("Failed to load input files")?;
            let module_root = args.module_root.clone().unwrap_or_else(|| ".".to_string());

            // 🌐 Try to extract provider schema if possible
            setup_provider_schema(args.working_directory.as_deref())?;
            
            let mapping = map_resources_to_modules(&modules_file.modules, &plan_file);
            execute_or_print_imports(&mapping, &plan_file, args.dry_run, args.verbose, &module_root, args.working_directory.as_deref());
            
            Ok(())
        }
    }
}

/// Unit tests for the main application functionality
/// 
/// These tests verify error handling, command execution, and integration behavior
/// of the main application functions, particularly around terragrunt operations.
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::Once;
    use tempfile::TempDir;
    use terragrunt_import_from_plan::utils::{run_terragrunt_init, write_provider_schema};

    static INIT: Once = Once::new();

    /// **TEST HELPER** - Ensures tests run in a clean state
    /// 
    /// This function uses a `Once` to ensure setup code only runs once per test session,
    /// providing a consistent testing environment.
    fn setup() {
        INIT.call_once(|| {
            // Setup code here
        });
    }

    /// **TEST** - Verifies terragrunt init behavior in a temporary directory
    /// 
    /// Tests that running `terragrunt init` in an empty temporary directory
    /// fails gracefully with an appropriate error. This is expected behavior
    /// since there's no terragrunt configuration in the temporary directory.
    #[test]
    fn test_01_setup_and_init() {
        setup();
        let temp_dir = TempDir::new().unwrap();
        let result = run_terragrunt_init(temp_dir.path().to_str().unwrap());
        assert!(result.is_err());
    }

    /// **TEST** - Verifies error handling for non-existent directories
    /// 
    /// Tests that running `terragrunt init` in a non-existent directory
    /// returns an appropriate error rather than panicking or succeeding.
    #[test]
    fn test_02_init_invalid_dir() {
        setup();
        let result = run_terragrunt_init("/nonexistent/path");
        assert!(result.is_err());
    }

    /// **TEST** - Verifies provider schema generation error handling
    /// 
    /// Tests that attempting to write provider schema in a directory without
    /// terragrunt configuration fails gracefully with an appropriate error.
    #[test]
    fn test_03_write_provider_schema() {
        setup();
        let temp_dir = TempDir::new().unwrap();
        let result = write_provider_schema(temp_dir.path());
        assert!(result.is_err());
    }

    /// **TEST** - Verifies error handling for invalid directory paths
    /// 
    /// Tests that attempting to write provider schema to a non-existent
    /// directory returns an appropriate error.
    #[test]
    fn test_04_write_provider_schema_invalid_dir() {
        setup();
        let result = write_provider_schema(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    /// **TEST** - Verifies command not found error handling
    /// 
    /// Tests that attempting to run a non-existent command returns the expected
    /// OS error code (ENOENT) rather than a different error type.
    #[test]
    fn test_05_write_provider_schema_terragrunt_not_found() {
        setup();
        let temp_dir = TempDir::new().unwrap();
        let result = Command::new("nonexistent_command")
            .arg("providers")
            .arg("schema")
            .arg("-json")
            .current_dir(temp_dir.path())
            .output();
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.raw_os_error().unwrap(), 2); // ENOENT
        } else {
            panic!("Expected error");
        }
    }

    /// **TEST** - Verifies error message formatting and context
    /// 
    /// Tests that error messages contain appropriate context information,
    /// including the command name and directory path. This ensures users
    /// get helpful error messages for debugging.
    #[test]
    fn test_06_error_context_formatting() {
        setup();
        let temp_dir = TempDir::new().unwrap();
        let result = run_terragrunt_init(temp_dir.path().to_str().unwrap());
        assert!(result.is_err());
        
        let error_string = format!("{:#}", result.unwrap_err());
        
        // Verify the error contains useful context
        // The error might be about terragrunt command not found rather than execution failure
        assert!(error_string.contains("terragrunt") && (
            error_string.contains("Failed to execute") || 
            error_string.contains("Terragrunt init failed") ||
            error_string.contains("command not found") ||
            error_string.contains("No such file")
        ));
        assert!(error_string.contains(temp_dir.path().to_str().unwrap()));
    }
}
