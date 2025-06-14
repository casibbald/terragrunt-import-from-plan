#!/bin/bash
set -euo pipefail

MODULES=(
  iam networking storage artifact_registry pubsub bigquery cloud_sql gke
  cloud_functions cloud_run monitoring logging secret_manager kms composer
  dataproc workflows spanner
)

mkdir -p modules

for module in "${MODULES[@]}"; do
  dir="modules/$module"
  echo "Creating module: $dir"
  mkdir -p "$dir"
  touch "$dir/main.tf" "$dir/variables.tf" "$dir/outputs.tf" "$dir/locals.tf"
done

echo "✅ Module skeletons initialized."
