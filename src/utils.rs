use crate::importer::{PlannedModule, Resource};
use serde_json::Value;
use std::collections::HashSet;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

pub use crate::schema::write_provider_schema;

#[derive(Error, Debug)]
pub enum TerragruntProcessError {
    #[error("Failed to run terragrunt: status={status}, stdout={stdout}, stderr={stderr}")]
    ProcessError {
        status: i32,
        stdout: String,
        stderr: String,
    },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Collects all resources from a planned module and its child modules recursively.
/// This is the consolidated function that replaces both collect_resources and collect_all_resources.
pub fn collect_resources<'a>(module: &'a PlannedModule, resources: &mut Vec<&'a Resource>) {
    if let Some(module_resources) = &module.resources {
        resources.extend(module_resources.iter());
    }
    if let Some(children) = &module.child_modules {
        for child in children {
            collect_resources(child, resources);
        }
    }
}



pub fn extract_id_candidate_fields(schema_json: &Value) -> HashSet<String> {
    let mut candidates = HashSet::new();

    if let Some(provider_schemas) = schema_json
        .get("provider_schemas")
        .and_then(|ps| ps.as_object())
    {
        // Iterate through all providers (aws, google, azurerm, etc.)
        for (_provider_name, provider_schema) in provider_schemas {
            if let Some(resource_schemas) = provider_schema
                .get("resource_schemas")
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
        }
    }

    candidates
}

pub fn run_terragrunt_init(working_directory: &str) -> Result<()> {
    println!("🔧 Running `terragrunt init` in {}", working_directory);

    let output = Command::new("terragrunt")
        .arg("init")
        .current_dir(working_directory)
        .output()
        .with_context(|| format!("Failed to execute terragrunt init in directory: {}", working_directory))?;

    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Terragrunt init failed in {}\nStatus: {}\nStdout: {}\nStderr: {}",
            working_directory,
            output.status.code().unwrap_or(-1),
            stdout.trim(),
            stderr.trim()
        );
    }
}

pub fn clean_workspace(provider: Option<&str>) -> Result<()> {
    fn remove_dirs_by_name(root: &Path, dir_name: &str) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().map(|n| n == dir_name).unwrap_or(false) {
                        let _ = fs::remove_dir_all(&path);
                    } else {
                        remove_dirs_by_name(&path, dir_name);
                    }
                }
            }
        }
    }

    fn remove_files_by_ext(root: &Path, ext: &str) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    remove_files_by_ext(&path, ext);
                } else if path.extension().map(|e| e == ext).unwrap_or(false) {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }

    fn remove_files_by_pattern(root: &Path, pattern: &str) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    remove_files_by_pattern(&path, pattern);
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(pattern) {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }

    fn remove_files_by_name(root: &Path, file_name: &str) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    remove_files_by_name(&path, file_name);
                } else if path.file_name().map(|n| n == file_name).unwrap_or(false) {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }

    let root = Path::new(".");
    remove_dirs_by_name(root, ".terraform");
    remove_dirs_by_name(root, ".terragrunt-cache");
    remove_files_by_ext(root, "tfstate");
    remove_files_by_pattern(root, ".lock.hcl");
    remove_files_by_name(root, "out.tfplan");

    // Clean provider-specific files for all providers or specific provider
    let providers = if let Some(p) = provider {
        vec![p]
    } else {
        vec!["aws", "gcp", "azure"]
    };

    for provider_name in providers {
        let env_path = Path::new("envs/simulator").join(provider_name).join("dev");
        if env_path.exists() {
            remove_files_by_name(&env_path, "plan.json");
            remove_files_by_name(&env_path, ".terragrunt-provider-schema.json");
        }
    }

    Ok(())
}

fn create_minimal_plan_json(provider: &str) -> Result<()> {
    let minimal_plan = r#"{"format_version":"1.2","terraform_version":"1.9.8","planned_values":{"root_module":{"child_modules":[]}}}"#;
    let fixtures_dir = format!("tests/fixtures/{}", provider);
    fs::create_dir_all(&fixtures_dir)?;
    fs::write(format!("{}/out.json", fixtures_dir), minimal_plan)?;
    println!("⚠️ Created minimal out.json for {} provider (plan failed)", provider);
    Ok(())
}

pub fn generate_fixtures(provider: &str) -> Result<()> {
    println!("🔧 Generating fixtures for {} provider...", provider);
    
    // Clean workspace for this provider
    clean_workspace(Some(provider))?;

    let env_path = format!("envs/simulator/{}/dev", provider);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    // Init
    println!("🚀 Running terragrunt init for {}...", provider);
    let init_output = Command::new("terragrunt")
        .arg("init")
        .arg("--all")
        .current_dir(&env_path)
        .output()
        .with_context(|| format!("Failed to run terragrunt init for {}", provider))?;

    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        eprintln!("⚠️ Warning: terragrunt init failed for {}: {}", provider, stderr);
        // Continue despite init failure (expected in CI)
    }

    // Plan
    println!("📋 Running terragrunt plan for {}...", provider);
    let plan_output = Command::new("terragrunt")
        .arg("run-all")
        .arg("plan")
        .arg("-out")
        .arg("out.tfplan")
        .current_dir(&env_path)
        .output()
        .with_context(|| format!("Failed to run terragrunt plan for {}", provider))?;

    // Always generate fixture files, even if plan fails
    let cache_path = format!("{}/.terragrunt-cache", env_path);
    generate_modules_json(provider, &cache_path)?;
    
    if !plan_output.status.success() {
        let stderr = String::from_utf8_lossy(&plan_output.stderr);
        eprintln!("⚠️ Warning: terragrunt plan failed for {}: {}", provider, stderr);
        // Create minimal plan file since plan failed
        create_minimal_plan_json(provider)?;
    } else {
        generate_plan_json(provider, &cache_path)?;
    }

    println!("✅ Fixtures generated successfully for {} provider", provider);
    Ok(())
}

fn generate_modules_json(provider: &str, cache_path: &str) -> Result<()> {
    // Extract module information and create modules.json
    let modules_json = format!(
        r#"{{"Modules":[{{"Key":"","Source":"","Dir":"."}}{}]}}"#,
        // Add modules based on actual directory structure
        if Path::new(&format!("simulator/{}/modules", provider)).exists() {
            let mut modules = Vec::new();
            if let Ok(entries) = fs::read_dir(&format!("simulator/{}/modules", provider)) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(module_name) = entry.file_name().to_str() {
                            modules.push(format!(
                                r#",{{"Key":"{}","Source":"./modules/{}","Dir":"modules/{}"}}"#,
                                module_name, module_name, module_name
                            ));
                        }
                    }
                }
            }
            modules.join("")
        } else {
            String::new()
        }
    );

    let fixtures_dir = format!("tests/fixtures/{}", provider);
    fs::create_dir_all(&fixtures_dir)?;
    fs::write(format!("{}/modules.json", fixtures_dir), modules_json)?;
    
    Ok(())
}

fn generate_plan_json(provider: &str, cache_path: &str) -> Result<()> {
    // Find terraform plan files and convert to JSON
    if let Ok(entries) = fs::read_dir(cache_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.extension().map(|e| e == "tfplan").unwrap_or(false) {
                            // Use terragrunt show instead of terraform show
                            let output = Command::new("terragrunt")
                                .arg("show")
                                .arg("-json")
                                .arg(sub_path.to_str().unwrap())
                                .current_dir(&path)
                                .output();
                            
                            if let Ok(output) = output {
                                if output.status.success() {
                                    let fixtures_dir = format!("tests/fixtures/{}", provider);
                                    fs::create_dir_all(&fixtures_dir)?;
                                    fs::write(format!("{}/out.json", fixtures_dir), output.stdout)?;
                                    println!("✅ Generated out.json for {} provider", provider);
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // If no plan file found or conversion failed, create a minimal out.json
    let minimal_plan = r#"{"format_version":"1.2","terraform_version":"1.9.8","planned_values":{"root_module":{"child_modules":[]}}}"#;
    let fixtures_dir = format!("tests/fixtures/{}", provider);
    fs::create_dir_all(&fixtures_dir)?;
    fs::write(format!("{}/out.json", fixtures_dir), minimal_plan)?;
    println!("⚠️ Created minimal out.json for {} provider (plan conversion failed)", provider);
    
    Ok(())
}

pub fn validate_terraform_format(provider: &str) -> Result<()> {
    println!("📝 Checking Terraform formatting for {}...", provider);
    
    let simulator_path = format!("simulator/{}", provider);
    if !Path::new(&simulator_path).exists() {
        anyhow::bail!("Provider directory does not exist: {}", simulator_path);
    }

    let output = Command::new("terraform")
        .arg("fmt")
        .arg("-check")
        .arg("-recursive")
        .arg(&simulator_path)
        .output()
        .with_context(|| format!("Failed to run terraform fmt for {}", provider))?;

    if output.status.success() {
        println!("✅ Terraform formatting is correct for {}", provider);
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Terraform formatting check failed for {}\nStdout: {}\nStderr: {}",
            provider,
            stdout.trim(),
            stderr.trim()
        );
    }
}

pub fn validate_terraform_config(provider: &str) -> Result<()> {
    println!("✅ Running terraform validate for {}...", provider);
    
    let simulator_path = format!("simulator/{}", provider);
    if !Path::new(&simulator_path).exists() {
        anyhow::bail!("Provider directory does not exist: {}", simulator_path);
    }

    // First run terraform init -backend=false
    println!("🔧 Running terraform init for validation...");
    let init_output = Command::new("terraform")
        .arg("init")
        .arg("-backend=false")
        .current_dir(&simulator_path)
        .env("AWS_EC2_METADATA_DISABLED", "true")
        .output()
        .with_context(|| format!("Failed to run terraform init for {}", provider))?;

    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        anyhow::bail!("Terraform init failed for {}: {}", provider, stderr.trim());
    }

    // Then run terraform validate
    let validate_output = Command::new("terraform")
        .arg("validate")
        .current_dir(&simulator_path)
        .env("AWS_EC2_METADATA_DISABLED", "true")
        .output()
        .with_context(|| format!("Failed to run terraform validate for {}", provider))?;

    if validate_output.status.success() {
        println!("✅ Terraform validation passed for {}", provider);
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&validate_output.stdout);
        let stderr = String::from_utf8_lossy(&validate_output.stderr);
        anyhow::bail!(
            "Terraform validation failed for {}\nStdout: {}\nStderr: {}",
            provider,
            stdout.trim(),
            stderr.trim()
        );
    }
}

pub fn format_terraform_files(provider: &str, check_only: bool) -> Result<()> {
    let action = if check_only { "Checking" } else { "Fixing" };
    println!("🔧 {} Terraform formatting for {}...", action, provider);
    
    let simulator_path = format!("simulator/{}", provider);
    if !Path::new(&simulator_path).exists() {
        anyhow::bail!("Provider directory does not exist: {}", simulator_path);
    }

    let mut cmd = Command::new("terraform");
    cmd.arg("fmt");
    
    if check_only {
        cmd.arg("-check");
    }
    
    cmd.arg("-recursive").arg(&simulator_path);

    let output = cmd.output()
        .with_context(|| format!("Failed to run terraform fmt for {}", provider))?;

    if output.status.success() {
        if check_only {
            println!("✅ Terraform formatting is correct for {}", provider);
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("🔧 Formatted files for {}:\n{}", provider, stdout.trim());
            } else {
                println!("✅ No formatting changes needed for {}", provider);
            }
        }
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if check_only {
            anyhow::bail!(
                "Terraform formatting check failed for {}\nFiles need formatting:\n{}\n{}",
                provider,
                stdout.trim(),
                stderr.trim()
            );
        } else {
            anyhow::bail!(
                "Terraform format failed for {}\nStdout: {}\nStderr: {}",
                provider,
                stdout.trim(),
                stderr.trim()
            );
        }
    }
}

pub fn init_terragrunt(provider: &str, env: &str, safe_mode: bool) -> Result<()> {
    println!("🚀 Initializing terragrunt for {} (env: {})...", provider, env);
    
    let env_path = format!("envs/simulator/{}/{}", provider, env);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    // First clean the workspace
    clean_workspace(Some(provider))?;

    // Then run terragrunt init
    let output = Command::new("terragrunt")
        .arg("init")
        .arg("--all")
        .current_dir(&env_path)
        .output()
        .with_context(|| format!("Failed to run terragrunt init for {}", provider))?;

    if output.status.success() {
        println!("✅ Terragrunt init succeeded for {}", provider);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = format!("Terragrunt init failed for {}: {}", provider, stderr.trim());
        
        if safe_mode {
            eprintln!("⚠️ Warning (safe mode): {}", error_msg);
            Ok(())
        } else {
            anyhow::bail!(error_msg);
        }
    }
}

pub fn plan_terragrunt(provider: &str, env: &str, vars: Option<&str>, safe_mode: bool) -> Result<()> {
    println!("📋 Planning terragrunt for {} (env: {})...", provider, env);
    
    let env_path = format!("envs/simulator/{}/{}", provider, env);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    let mut cmd = Command::new("terragrunt");
    cmd.arg("run-all")
        .arg("plan")
        .arg("-out")
        .arg("out.tfplan")
        .current_dir(&env_path)
        .env("AWS_EC2_METADATA_DISABLED", "true");

    // Add environment variables if provided
    if let Some(vars_str) = vars {
        for var_pair in vars_str.split_whitespace() {
            if let Some((key, value)) = var_pair.split_once('=') {
                cmd.env(key, value);
            }
        }
    }

    let output = cmd.output()
        .with_context(|| format!("Failed to run terragrunt plan for {}", provider))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("✅ Terragrunt plan succeeded for {}", provider);
        
        // Try to extract plan summary
        if let Some(plan_line) = stdout.lines().find(|line| line.contains("Plan:")) {
            println!("📊 {}", plan_line.trim());
        }
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = format!("Terragrunt plan failed for {}: {}", provider, stderr.trim());
        
        if safe_mode {
            eprintln!("⚠️ Warning (safe mode): {}", error_msg);
            Ok(())
        } else {
            anyhow::bail!(error_msg);
        }
    }
}

pub fn apply_terragrunt(provider: &str, env: &str, safe_mode: bool) -> Result<()> {
    println!("🚀 Applying terragrunt for {} (env: {})...", provider, env);
    
    let env_path = format!("envs/simulator/{}/{}", provider, env);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    let output = Command::new("terragrunt")
        .arg("run-all")
        .arg("apply")
        .current_dir(&env_path)
        .output()
        .with_context(|| format!("Failed to run terragrunt apply for {}", provider))?;

    if output.status.success() {
        println!("✅ Terragrunt apply succeeded for {}", provider);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = format!("Terragrunt apply failed for {}: {}", provider, stderr.trim());
        
        if safe_mode {
            eprintln!("⚠️ Warning (safe mode): {}", error_msg);
            Ok(())
        } else {
            anyhow::bail!(error_msg);
        }
    }
}

pub fn destroy_terragrunt(provider: &str, env: &str, safe_mode: bool) -> Result<()> {
    println!("💥 Destroying terragrunt infrastructure for {} (env: {})...", provider, env);
    
    let env_path = format!("envs/simulator/{}/{}", provider, env);
    if !Path::new(&env_path).exists() {
        anyhow::bail!("Environment path does not exist: {}", env_path);
    }

    let output = Command::new("terragrunt")
        .arg("run-all")
        .arg("destroy")
        .current_dir(&env_path)
        .output()
        .with_context(|| format!("Failed to run terragrunt destroy for {}", provider))?;

    if output.status.success() {
        println!("✅ Terragrunt destroy succeeded for {}", provider);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = format!("Terragrunt destroy failed for {}: {}", provider, stderr.trim());
        
        if safe_mode {
            eprintln!("⚠️ Warning (safe mode): {}", error_msg);
            Ok(())
        } else {
            anyhow::bail!(error_msg);
        }
    }
}
