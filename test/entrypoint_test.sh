#!/bin/bash
set -euo pipefail

mkdir -p test/tmp && cd test/tmp

# Create a valid plan.json directly
cat > plan.json <<EOF
{
  "resource_changes": [
    {
      "address": "google_artifact_registry_repository.remote_repos[\"mock-repo\"]",
      "change": {
        "actions": ["create"],
        "after": {
          "repository_id": "mock-repo"
        }
      }
    }
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

# Skip terraform show; pretend it's already parsed
cp plan.json tf.plan

# Patch entrypoint to use plan.json directly
sed 's/terraform show -json tf.plan > plan.json/# using plan.json from test/' ../../entrypoint.sh | bash

cd ../..
