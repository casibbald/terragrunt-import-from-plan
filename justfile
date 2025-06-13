# Default variables
default_env := env_var_or_default("ENV", "dev")
default_region := env_var_or_default("REGION", "us-central1")
justfile_dir := env_var_or_default("JUSTFILE_DIR", ".")
default_project := env_var_or_default("PROJECT_ID", "my-gcp-project")
terragrunt_dir := env_var_or_default("TERRAGRUNT_DIR", "envs/simulator/")

# List available commands
default:
    @just --list

run:
    cargo run -- --plan tests/fixtures/out.json --modules tests/fixtures/modules.json --module-root simulator/modules --dry-run

gen:
    just clean
    just init
    just plan
    just plans-to-json
    just copy-plan-json


# Initialize all modules
init env=default_env:
    just clean
    cd {{terragrunt_dir}}/{{env}} && terragrunt init --all

# Plan all modules
# Plan all modules
# Do not give a .tf to the binary output, it will not work with Terragrunt
plan env=default_env *VARS="":
    cd {{terragrunt_dir}}/{{env}} && {{VARS}} terragrunt run-all plan -out out.tfplan


# Convert all .tfplan files to plan.json in-place under .terragrunt-cache
plans-to-json env=default_env *VARS="":
    cd envs/simulator/dev && \
    find .terragrunt-cache -type f -name '*.tfplan' | while read plan; do \
      echo "Converting $plan to JSON..."; \
      terraform -chdir="$(dirname "$plan")" show -json "$(basename "$plan")" | jq '.' > "test/tmp/$(basename "$plan" .tfplan).json"; \
    done

copy-plan-json env=default_env *VARS="":
    cd envs/simulator/dev && \
    find ./.terragrunt-cache -name "*.json" -type f -exec ls {} \; | while read plan; do \
      echo "Copying $plan to test/tmp..."; \
      cp "$plan" "../../../tests/fixtures/$(basename "$plan")"; \
    done





# Apply all modules
apply env=default_env:
    cd {{terragrunt_dir}}/{{env}} && terragrunt run-all apply

# Destroy all infrastructure
destroy env=default_env:
    cd {{terragrunt_dir}}/{{env}} && terragrunt run-all destroy

# Module-specific commands
plan-module module env=default_env:
    cd {{terragrunt_dir}}/{{env}}/{{module}} && terragrunt plan -out ../../../test/tmp/{{module}}.tf

apply-module module env=default_env:
    cd {{terragrunt_dir}}/{{env}}/{{module}} && terragrunt apply

# Validate Terraform files. Do not run this, there are no Terraform files in this project.
# Additionally do not run in production environments.
validate:
    find . -name "*.tf" -exec terraform fmt -check {} \;
    find . -name "*.tf" -exec terraform validate {} \;

# Clean Terraform cache and state files (use with caution)
clean:
    find . -name ".terraform" -type d -exec rm -rf {} +
    find . -name ".terragrunt-cache" -type d -exec rm -rf {} +
    find . -name "*.tfstate" -type f -exec rm -f {} +
    find . -name ".*.lock.hcl" -type f -exec rm -f {} +
    find . -name "out.tfplan" -type f -exec rm -f {} +
    find envs/simulator/dev -name "plan.json" -type f -exec rm -f {} +
    find envs/simulator/dev -name ".terragrunt-provider-schema.json" -type f -exec rm -f {} +

test:
    cargo test -- --test-threads=1
