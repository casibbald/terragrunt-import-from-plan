# Default variables
default_env := env_var_or_default("ENV", "dev")
default_region := env_var_or_default("REGION", "us-central1")
justfile_dir := env_var_or_default("JUSTFILE_DIR", ".")
default_project := env_var_or_default("PROJECT_ID", "my-gcp-project")
default_cloud := env_var_or_default("CLOUD", "gcp")
terragrunt_dir := env_var_or_default("TERRAGRUNT_DIR", "envs/simulator/" + default_cloud + "/")

# List available commands
default:
    @just --list

run cloud=default_cloud:
    cargo run -- --plan tests/fixtures/{{cloud}}/out.json --modules tests/fixtures/{{cloud}}/modules.json --module-root simulator/{{cloud}}/modules --dry-run

gen cloud=default_cloud:
    cargo run -- generate-fixtures {{cloud}}


# Initialize all modules
init cloud=default_cloud env=default_env:
    cargo run -- init {{cloud}} --env {{env}}

# Plan all modules
plan cloud=default_cloud env=default_env *VARS="":
    cargo run -- plan {{cloud}} --env {{env}} --vars "{{VARS}}"


# Convert all .tfplan files to plan.json in-place under .terragrunt-cache (replaced by generate-fixtures)
plans-to-json cloud=default_cloud env=default_env *VARS="":
    cargo run -- generate-fixtures {{cloud}}

copy-plan-json cloud=default_cloud env=default_env *VARS="":
    cargo run -- generate-fixtures {{cloud}}


# Apply all modules
apply cloud=default_cloud env=default_env:
    cargo run -- apply {{cloud}} --env {{env}}

# Destroy all infrastructure
destroy cloud=default_cloud env=default_env:
    cargo run -- destroy {{cloud}} --env {{env}}

# Module-specific commands
plan-module module cloud=default_cloud env=default_env:
    cd envs/simulator/{{cloud}}/{{env}}/{{module}} && terragrunt plan -out ../../../test/tmp/{{module}}.tf

apply-module module cloud=default_cloud env=default_env:
    cd envs/simulator/{{cloud}}/{{env}}/{{module}} && terragrunt apply

# Comprehensive Terraform validation for all cloud providers  
validate cloud=default_cloud:
    cargo run -- validate {{cloud}}

# Run all validations for all cloud providers for all cloud providers (AWS, GCP, Azure)
validate-all:
    cargo run -- validate aws
    cargo run -- validate gcp  
    cargo run -- validate azure

# Fix formatting issues
fmt cloud=default_cloud:
    cargo run -- fmt {{cloud}}

# Fix formatting for all cloud providers for all cloud providers (AWS, GCP, Azure)
fmt-all:
    cargo run -- fmt aws
    cargo run -- fmt gcp
    cargo run -- fmt azure

# CI-friendly validation (no API calls, no credentials needed) for all cloud providers (AWS, GCP, Azure)
validate-ci:
    cargo run -- validate aws
    cargo run -- validate gcp  
    cargo run -- validate azure

# Clean Terraform cache and state files (use with caution)
clean cloud=default_cloud:
    cargo run -- clean {{cloud}}

test:
    cargo test -- --test-threads=1

# Plan with error handling for testing (continues on failure)
plan-safe cloud=default_cloud env=default_env *VARS="":
    cargo run -- plan {{cloud}} --env {{env}} --vars "{{VARS}}" --safe

# Initialize with error handling for testing (continues on failure)  
init-safe cloud=default_cloud env=default_env:
    cargo run -- init {{cloud}} --env {{env}} --safe

# Run tests with fresh provider schemas for all cloud providers (AWS, GCP, Azure)
test-all:
    cargo build
    cargo run -- clean gcp
    cargo run -- clean aws
    cargo run -- clean azure
    cargo run -- init gcp --safe
    cargo run -- init aws --safe
    cargo run -- init azure --safe
    cargo run -- plan gcp --safe
    cargo run -- plan aws --safe
    cargo run -- plan azure --safe
    cargo test -- --test-threads=1 -- --nocapture

# Multi-cloud convenience commands
run-gcp:
    cargo run -- --plan tests/fixtures/gcp/out.json --modules tests/fixtures/gcp/modules.json --module-root simulator/gcp/modules --dry-run

run-aws:
    cargo run -- --plan tests/fixtures/aws/out.json --modules tests/fixtures/aws/modules.json --module-root simulator/aws/modules --dry-run

run-azure:
    cargo run -- --plan tests/fixtures/azure/out.json --modules tests/fixtures/azure/modules.json --module-root simulator/azure/modules --dry-run

gen-gcp:
    cargo run -- generate-fixtures gcp

gen-aws:
    cargo run -- generate-fixtures aws

gen-azure:
    cargo run -- generate-fixtures azure

init-gcp:
    cargo run -- init gcp

init-aws:
    cargo run -- init aws

init-azure:
    cargo run -- init azure

# for all cloud providers (AWS, GCP, Azure)
clean-all:
    cargo run -- clean gcp
    cargo run -- clean aws
    cargo run -- clean azure
