#!/bin/bash
set -xeuo pipefail

mkdir -p test/tmp && cd test/tmp

# Create a realistic plan JSON for our Rust binary to parse
cat > plan.json <<EOF
{
  "format_version": "1.0",
  "terraform_version": "1.0",
  "variables": null,
  "planned_values": {
    "root_module": {
      "child_modules": [
        {
          "address": "module.artifact_registry",
          "resources": [
            {
              "address": "module.artifact_registry.google_artifact_registry_repository.remote_repos[\"mock-repo\"]",
              "mode": "managed",
              "type": "google_artifact_registry_repository", 
              "name": "remote_repos",
              "values": {
                "repository_id": "mock-repo",
                "name": "mock-repo"
              }
            }
          ]
        }
      ]
    }
  },
  "provider_schemas": null
}
EOF

# Create modules.json
cat > modules.json <<EOF
{
  "Modules": [
    {"Key": "", "Source": "", "Dir": "."},
    {"Key": "artifact_registry", "Source": "./modules/artifact_registry", "Dir": "modules/artifact_registry"}
  ]
}
EOF

# Mock terragrunt for test purposes
terragrunt() {
  echo "MOCK: terragrunt $@" >&2
  if [[ "$1" == "state" ]]; then
    return 1  # Simulate not-in-state
  fi
  return 0
}
export -f terragrunt

# Mock terraform command for plan conversion (shouldn't be called with .json input)
terraform() {
  echo "MOCK: terraform $@" >&2
  return 0
}
export -f terraform

# Mock cargo build (assume binary exists)
cargo() {
  if [[ "$1" == "build" ]]; then
    echo "MOCK: cargo $@" >&2
    # Create a mock binary that will use our actual binary
    mkdir -p target/release
    cp ../../target/debug/terragrunt_import_from_plan target/release/ 2>/dev/null || true
    return 0
  fi
  echo "MOCK: cargo $@" >&2
  return 0
}
export -f cargo

# Set environment variables for dry run
export DRY_RUN=true
export VERBOSE=true

# Run entrypoint with JSON input and capture output
output=$(bash ../../entrypoint.sh plan.json modules.json 2>&1)

echo "$output"

# Assertions for new Rust-based output
if ! grep -q "Imported:   1" <<< "$output"; then
  echo "❌ Test failed: Expected 1 import."
  exit 1
fi

if ! grep -q "module.artifact_registry.google_artifact_registry_repository.remote_repos" <<< "$output"; then
  echo "❌ Test failed: Expected resource address in output."
  exit 1
fi

echo "✅ Test passed."
cd ../..
