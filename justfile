# Default variables
# Default variables
default_env := env_var_or_default("ENV", "dev")
default_region := env_var_or_default("REGION", "us-central1")
default_project := env_var_or_default("PROJECT_ID", "my-gcp-project")
terragrunt_dir := env_var_or_default("TERRAGRUNT_DIR", "envs/simulator/")

# List available commands
default:
    @just --list

# Initialize all modules
init env=default_env:
    cd {{terragrunt_dir}}/{{env}} && terragrunt run-all init

# Plan all modules
# Plan all modules
plan env=default_env *VARS="":
    cd {{terragrunt_dir}}/{{env}} && {{VARS}} terragrunt run-all plan

# Apply all modules
apply env=default_env:
    cd {{terragrunt_dir}}/{{env}} && terragrunt run-all apply

# Destroy all infrastructure
destroy env=default_env:
    cd {{terragrunt_dir}}/{{env}} && terragrunt run-all destroy

# Module-specific commands
plan-module module env=default_env:
    cd {{terragrunt_dir}}/{{env}}/{{module}} && terragrunt plan

apply-module module env=default_env:
    cd {{terragrunt_dir}}/{{env}}/{{module}} && terragrunt apply

# Validate Terraform files
validate:
    find . -name "*.tf" -exec terraform fmt -check {} \;
    find . -name "*.tf" -exec terraform validate {} \;

# Clean Terraform cache and state files (use with caution)
clean:
    find . -name ".terraform" -type d -exec rm -rf {} +
    find . -name ".terragrunt-cache" -type d -exec rm -rf {} +