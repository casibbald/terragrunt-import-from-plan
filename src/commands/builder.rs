use std::path::{Path, PathBuf};
use crate::importer::{ModuleMeta, ResourceWithId};
use super::ImportCommand;

/// Builder for creating terragrunt import commands
pub struct ImportCommandBuilder {
    module_root: PathBuf,
}

impl ImportCommandBuilder {
    /// Create a new ImportCommandBuilder with the specified module root
    pub fn new<P: AsRef<Path>>(module_root: P) -> Self {
        Self {
            module_root: module_root.as_ref().to_path_buf(),
        }
    }

    /// Build a single import command for a resource
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

    /// Build import commands for multiple resources
    pub fn build_all_commands(&self, resources: &[ResourceWithId]) -> Vec<ImportCommand> {
        resources
            .iter()
            .map(|resource| self.build_command(resource, resource.module_meta))
            .collect()
    }

    /// Build a command string for dry-run display
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