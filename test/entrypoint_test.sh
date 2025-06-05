#!/bin/bash
set -xeuo pipefail

mkdir -p test/tmp && cd test/tmp

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

# Run entrypoint with JSON input
bash ../../entrypoint.sh plan.json

cd ../..
