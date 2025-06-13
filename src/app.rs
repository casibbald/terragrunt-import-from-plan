use crate::importer::{ModulesFile, PlanFile};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Loads and parses the modules file from the given path
pub fn load_modules<P: AsRef<Path>>(path: P) -> Result<ModulesFile> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read modules file: {}", path.display()))?;
    
    let modules: ModulesFile = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse modules JSON in file: {}", path.display()))?;
    
    Ok(modules)
}

/// Loads and parses the plan file from the given path
pub fn load_plan<P: AsRef<Path>>(path: P) -> Result<PlanFile> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read plan file: {}", path.display()))?;
    
    let plan: PlanFile = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse plan JSON in file: {}", path.display()))?;
    
    Ok(plan)
}

/// Loads both modules and plan files with proper error context
pub fn load_input_files<P1: AsRef<Path>, P2: AsRef<Path>>(
    modules_path: P1, 
    plan_path: P2
) -> Result<(ModulesFile, PlanFile)> {
    let modules = load_modules(modules_path)
        .context("Failed to load modules file")?;
    
    let plan = load_plan(plan_path)
        .context("Failed to load plan file")?;
    
    Ok((modules, plan))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_modules_success() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"Modules": [{{"Key": "test", "Source": "./test", "Dir": "test"}}]}}"#
        ).unwrap();
        
        let result = load_modules(temp_file.path());
        assert!(result.is_ok());
        
        let modules = result.unwrap();
        assert_eq!(modules.modules.len(), 1);
        assert_eq!(modules.modules[0].key, "test");
    }

    #[test]
    fn test_load_modules_file_not_found() {
        let result = load_modules("/nonexistent/path/modules.json");
        assert!(result.is_err());
        let error_string = result.unwrap_err().to_string();
        assert!(error_string.contains("Failed to read modules file"));
    }

    #[test]
    fn test_load_modules_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid json").unwrap();
        
        let result = load_modules(temp_file.path());
        assert!(result.is_err());
        let error_string = result.unwrap_err().to_string();
        assert!(error_string.contains("Failed to parse modules JSON"));
    }

    #[test]
    fn test_load_plan_success() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"format_version": "1.0", "terraform_version": "1.0", "variables": null, "planned_values": null, "provider_schemas": null}}"#
        ).unwrap();
        
        let result = load_plan(temp_file.path());
        assert!(result.is_ok());
        
        let plan = result.unwrap();
        assert_eq!(plan.format_version, "1.0");
    }

    #[test]
    fn test_load_plan_file_not_found() {
        let result = load_plan("/nonexistent/path/plan.json");
        assert!(result.is_err());
        let error_string = result.unwrap_err().to_string();
        assert!(error_string.contains("Failed to read plan file"));
    }

    #[test]
    fn test_load_plan_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid json").unwrap();
        
        let result = load_plan(temp_file.path());
        assert!(result.is_err());
        let error_string = result.unwrap_err().to_string();
        assert!(error_string.contains("Failed to parse plan JSON"));
    }

    #[test]
    fn test_load_input_files_success() {
        let mut modules_file = NamedTempFile::new().unwrap();
        writeln!(
            modules_file,
            r#"{{"Modules": [{{"Key": "test", "Source": "./test", "Dir": "test"}}]}}"#
        ).unwrap();
        
        let mut plan_file = NamedTempFile::new().unwrap();
        writeln!(
            plan_file,
            r#"{{"format_version": "1.0", "terraform_version": "1.0", "variables": null, "planned_values": null, "provider_schemas": null}}"#
        ).unwrap();
        
        let result = load_input_files(modules_file.path(), plan_file.path());
        assert!(result.is_ok());
        
        let (modules, plan) = result.unwrap();
        assert_eq!(modules.modules.len(), 1);
        assert_eq!(plan.format_version, "1.0");
    }

    #[test]
    fn test_load_input_files_modules_fail() {
        let mut plan_file = NamedTempFile::new().unwrap();
        writeln!(
            plan_file,
            r#"{{"format_version": "1.0", "terraform_version": "1.0", "variables": null, "planned_values": null, "provider_schemas": null}}"#
        ).unwrap();
        
        let result = load_input_files("/nonexistent/modules.json", plan_file.path());
        assert!(result.is_err());
        let error_string = result.unwrap_err().to_string();
        assert!(error_string.contains("Failed to load modules file"));
    }
} 