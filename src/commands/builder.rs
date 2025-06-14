//! # Import Command Builder Module
//! 
//! This module provides functionality for building terragrunt import commands from
//! processed resource information. It handles the construction of both structured
//! command objects and formatted command strings for execution or display.
//! 
//! ## Key Functionality
//! 
//! - **Command Construction**: Build structured ImportCommand objects
//! - **Batch Processing**: Generate commands for multiple resources
//! - **Path Resolution**: Handle module directory path resolution
//! - **String Formatting**: Generate formatted command strings for display
//! 
//! ## Usage Pattern
//! 
//! 1. Create an ImportCommandBuilder with the module root directory
//! 2. Use build_command() for individual resources or build_all_commands() for batches
//! 3. Commands can be executed via the executor module or displayed as strings

use std::path::{Path, PathBuf};
use crate::importer::{ModuleMeta, ResourceWithId};
use super::ImportCommand;

/// Builder for creating terragrunt import commands
/// 
/// This builder provides a convenient interface for constructing terragrunt import
/// commands from processed resources. It handles path resolution, command formatting,
/// and batch operations for multiple resources.
/// 
/// # Fields
/// - `module_root`: Base directory for resolving module paths
/// 
/// # Examples
/// ```no_run
/// use terragrunt_import_from_plan::commands::builder::ImportCommandBuilder;
/// 
/// let builder = ImportCommandBuilder::new("./modules");
/// // Use builder to create import commands...
/// ```
pub struct ImportCommandBuilder {
    /// Base directory for resolving relative module paths
    module_root: PathBuf,
}

impl ImportCommandBuilder {
    /// Creates a new ImportCommandBuilder with the specified module root directory
    /// 
    /// The module root is used to resolve relative paths in module metadata to
    /// absolute paths for command execution. This should typically be the root
    /// directory of your terragrunt workspace.
    /// 
    /// # Arguments
    /// * `module_root` - Base directory for resolving module paths
    /// 
    /// # Returns
    /// A new ImportCommandBuilder instance ready for command generation
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::commands::builder::ImportCommandBuilder;
    /// 
    /// let builder = ImportCommandBuilder::new("/path/to/terragrunt/workspace");
    /// let builder2 = ImportCommandBuilder::new("./relative/path");
    /// ```
    pub fn new<P: AsRef<Path>>(module_root: P) -> Self {
        Self {
            module_root: module_root.as_ref().to_path_buf(),
        }
    }

    /// Builds a single terragrunt import command for a resource
    /// 
    /// This method constructs a complete ImportCommand object containing all the
    /// information needed to execute a terragrunt import operation. It resolves
    /// the full module path and extracts all necessary resource information.
    /// 
    /// # Arguments
    /// * `resource` - ResourceWithId containing the resource and inferred ID
    /// * `module` - ModuleMeta containing module information
    /// 
    /// # Returns
    /// ImportCommand object ready for execution
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::commands::builder::ImportCommandBuilder;
    /// 
    /// let builder = ImportCommandBuilder::new("./modules");
    /// let command = builder.build_command(&resource_with_id, &module_meta);
    /// ```
    pub fn build_command(&self, resource: &ResourceWithId, module: &ModuleMeta) -> ImportCommand {
        let full_path = self.module_root.join(&module.dir);
        
        ImportCommand {
            working_directory: full_path,
            resource_address: resource.resource.address.clone(),
            resource_id: resource.id.clone(),
            resource_type: resource.resource.r#type.clone(),
            module_name: module.key.clone(),
        }
    }

    /// Builds import commands for multiple resources in batch
    /// 
    /// This method processes a collection of resources and generates ImportCommand
    /// objects for each one. This is more efficient than calling build_command()
    /// individually for many resources.
    /// 
    /// # Arguments
    /// * `resources` - Slice of ResourceWithId objects to generate commands for
    /// 
    /// # Returns
    /// Vector of ImportCommand objects, one for each input resource
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::commands::builder::ImportCommandBuilder;
    /// 
    /// let builder = ImportCommandBuilder::new("./modules");
    /// let commands = builder.build_all_commands(&resource_list);
    /// println!("Generated {} import commands", commands.len());
    /// ```
    pub fn build_all_commands(&self, resources: &[ResourceWithId]) -> Vec<ImportCommand> {
        resources
            .iter()
            .map(|resource| self.build_command(resource, resource.module_meta))
            .collect()
    }

    /// Builds a formatted command string for dry-run display
    /// 
    /// This method generates a human-readable command string that shows exactly
    /// what terragrunt command would be executed. This is useful for dry-run mode
    /// or for displaying commands to users before execution.
    /// 
    /// # Arguments
    /// * `resource` - ResourceWithId containing the resource and inferred ID
    /// 
    /// # Returns
    /// Formatted terragrunt import command string
    /// 
    /// # Command Format
    /// `terragrunt import -config-dir={module_path} {resource_address} {resource_id}`
    /// 
    /// # Examples
    /// ```no_run
    /// use terragrunt_import_from_plan::commands::builder::ImportCommandBuilder;
    /// 
    /// let builder = ImportCommandBuilder::new("./modules");
    /// let command_str = builder.build_command_string(&resource_with_id);
    /// println!("Would execute: {}", command_str);
    /// ```
    pub fn build_command_string(&self, resource: &ResourceWithId) -> String {
        let full_path = self.module_root.join(&resource.module_meta.dir);
        format!(
            "terragrunt import -config-dir={} {} {}",
            full_path.display(),
            resource.resource.address,
            resource.id
        )
    }
} 