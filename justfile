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
    just clean {{cloud}}
    just init {{cloud}}
    just plan {{cloud}}
    just plans-to-json {{cloud}}
    just copy-plan-json {{cloud}}


# Initialize all modules
init cloud=default_cloud env=default_env:
    just clean {{cloud}}
    cd envs/simulator/{{cloud}}/{{env}} && terragrunt init --all

# Plan all modules
# Do not give a .tf to the binary output, it will not work with Terragrunt
plan cloud=default_cloud env=default_env *VARS="":
    cd envs/simulator/{{cloud}}/{{env}} && AWS_EC2_METADATA_DISABLED=true {{VARS}} terragrunt run-all plan -out out.tfplan


# Convert all .tfplan files to plan.json in-place under .terragrunt-cache
plans-to-json cloud=default_cloud env=default_env *VARS="":
    cd envs/simulator/{{cloud}}/{{env}} && \
    find .terragrunt-cache -type f -name '*.tfplan' | while read plan; do \
      echo "Converting $plan to JSON..."; \
      terraform -chdir="$(dirname "$plan")" show -json "$(basename "$plan")" | jq '.' > "test/tmp/$(basename "$plan" .tfplan).json"; \
    done

copy-plan-json cloud=default_cloud env=default_env *VARS="":
    cd envs/simulator/{{cloud}}/{{env}} && \
    find ./.terragrunt-cache -name "*.json" -type f -exec ls {} \; | while read plan; do \
      echo "Copying $plan to test/tmp..."; \
      cp "$plan" "../../../tests/fixtures/$(basename "$plan")"; \
    done


# Apply all modules
apply cloud=default_cloud env=default_env:
    cd envs/simulator/{{cloud}}/{{env}} && terragrunt run-all apply

# Destroy all infrastructure
destroy cloud=default_cloud env=default_env:
    cd envs/simulator/{{cloud}}/{{env}} && terragrunt run-all destroy

# Module-specific commands
plan-module module cloud=default_cloud env=default_env:
    cd envs/simulator/{{cloud}}/{{env}}/{{module}} && terragrunt plan -out ../../../test/tmp/{{module}}.tf

apply-module module cloud=default_cloud env=default_env:
    cd envs/simulator/{{cloud}}/{{env}}/{{module}} && terragrunt apply

# Comprehensive Terraform validation for all cloud providers
validate cloud=default_cloud:
    echo "🔍 Running comprehensive validation for {{cloud}}..."
    just validate-format {{cloud}}
    just validate-terraform {{cloud}}

# Format validation
validate-format cloud=default_cloud:
    echo "📝 Checking Terraform formatting for {{cloud}}..."
    terraform fmt -check -recursive simulator/{{cloud}}/

# Terraform validate for specific cloud provider (no credentials needed)
validate-terraform cloud=default_cloud:
    echo "✅ Running terraform validate for {{cloud}}..."
    cd simulator/{{cloud}} && AWS_EC2_METADATA_DISABLED=true terraform init -backend=false
    cd simulator/{{cloud}} && AWS_EC2_METADATA_DISABLED=true terraform validate

# Run all validations for all cloud providers
validate-all:
    echo "🌐 Running validation for all cloud providers..."
    just validate aws
    just validate gcp  
    just validate azure

# Fix formatting issues
fmt cloud=default_cloud:
    echo "🔧 Fixing Terraform formatting for {{cloud}}..."
    terraform fmt -recursive simulator/{{cloud}}/

# Fix formatting for all cloud providers  
fmt-all:
    echo "🔧 Fixing formatting for all cloud providers..."
    just fmt aws
    just fmt gcp
    just fmt azure

# CI-friendly validation (no API calls, no credentials needed)
validate-ci:
    echo "🚀 Running CI-friendly validation..."
    AWS_EC2_METADATA_DISABLED=true just validate-all

# Clean Terraform cache and state files (use with caution)
clean cloud=default_cloud:
    find . -name ".terraform" -type d -exec rm -rf {} +
    find . -name ".terragrunt-cache" -type d -exec rm -rf {} +
    find . -name "*.tfstate" -type f -exec rm -f {} +
    find . -name ".*.lock.hcl" -type f -exec rm -f {} +
    find . -name "out.tfplan" -type f -exec rm -f {} +
    find envs/simulator/{{cloud}}/dev -name "plan.json" -type f -exec rm -f {} +
    find envs/simulator/{{cloud}}/dev -name ".terragrunt-provider-schema.json" -type f -exec rm -f {} +

test:
    cargo test -- --test-threads=1

# Plan with error handling for testing (continues on failure)
plan-safe cloud=default_cloud env=default_env *VARS="":
    -cd envs/simulator/{{cloud}}/{{env}} && AWS_EC2_METADATA_DISABLED=true {{VARS}} terragrunt run-all plan -out out.tfplan

# Initialize with error handling for testing (continues on failure)  
init-safe cloud=default_cloud env=default_env:
    -just clean {{cloud}}
    -cd envs/simulator/{{cloud}}/{{env}} && terragrunt init --all

# Run tests with fresh provider schemas for both AWS and GCP
test-with-fresh-schemas:
    just clean-all
    AWS_EC2_METADATA_DISABLED=true just init-safe gcp
    AWS_EC2_METADATA_DISABLED=true just init-safe aws
    AWS_EC2_METADATA_DISABLED=true just plan-safe gcp
    AWS_EC2_METADATA_DISABLED=true just plan-safe aws
    cargo test -- --test-threads=1

# Multi-cloud convenience commands
run-gcp:
    just run gcp

run-aws:
    just run aws

gen-gcp:
    just gen gcp

gen-aws:
    just gen aws

init-gcp:
    just init gcp

init-aws:
    just init aws

clean-all:
    just clean gcp
    just clean aws
