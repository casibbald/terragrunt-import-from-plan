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
    cd envs/simulator/{{cloud}}/{{env}} && {{VARS}} terragrunt run-all plan -out out.tfplan


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

# Validate Terraform files. Do not run this, there are no Terraform files in this project.
# Additionally do not run in production environments.
validate:
    find . -name "*.tf" -exec terraform fmt -check {} \;
    find . -name "*.tf" -exec terraform validate {} \;

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
    -cd envs/simulator/{{cloud}}/{{env}} && {{VARS}} terragrunt run-all plan -out out.tfplan

# Initialize with error handling for testing (continues on failure)  
init-safe cloud=default_cloud env=default_env:
    -just clean {{cloud}}
    -cd envs/simulator/{{cloud}}/{{env}} && terragrunt init --all

# Run tests with fresh provider schemas for both AWS and GCP
test-with-fresh-schemas:
    just clean-all
    just init-safe gcp
    just init-safe aws
    just plan-safe gcp
    just plan-safe aws
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
