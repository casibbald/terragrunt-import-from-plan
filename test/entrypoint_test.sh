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

# Run entrypoint with JSON input and capture output
output=$(bash ../../entrypoint.sh plan.json 2>&1)

echo "$output"

# Assertions
if ! grep -q "Imported:   1" <<< "$output"; then
  echo "❌ Test failed: Expected 1 import."
  exit 1
fi

if ! grep -q "MOCK: terragrunt import" <<< "$output"; then
  echo "❌ Test failed: Import command was not triggered."
  exit 1
fi

echo "✅ Test passed."
cd ../..
