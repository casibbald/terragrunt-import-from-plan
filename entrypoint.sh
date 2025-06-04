#!/bin/bash
set -euo pipefail

# Ensure the plan exists
if [[ ! -f tf.plan ]]; then
  echo "❌ Error: tf.plan not found. Run 'terragrunt plan -out=tf.plan' before this action."
  exit 1
fi

# Export plan to JSON
terraform show -json tf.plan > plan.json

imported=()
skipped=()
already=()

# Process each creatable resource
jq -c '
  .resource_changes[]
  | select(.change.actions | inside(["create"]) or inside(["create", "update"]) or inside(["create", "delete", "update"]))
  | {address: .address, after: .change.after}
' plan.json | while read -r line; do
  address=$(echo "$line" | jq -r '.address')
  after_obj=$(echo "$line" | jq '.after')

  id=$(echo "$after_obj" | jq -r '
    if has("name") then .name
    elif has("repository_id") then .repository_id
    elif has("bucket") then .bucket
    elif has("id") then .id
    else empty end
  ')

  if [[ -z "$id" ]]; then
    echo "⚠️ Skipping $address — no importable ID field found."
    skipped+=("$address")
    continue
  fi

  # Apply optional GCP prefix if provided
  if [[ -n "${PROJECT_ID:-}" && -n "${LOCATION:-}" ]]; then
    id="projects/${PROJECT_ID}/locations/${LOCATION}/repositories/${id}"
  fi

  echo "🔍 Checking $address..."
  if terragrunt state show "$address" > /dev/null 2>&1; then
    echo "✅ $address already in state"
    already+=("$address")
  else
    echo "📦 Importing $address with ID: $id"
    terragrunt import "$address" "$id"
    imported+=("$address")
  fi

done

# Summary
echo "\n✅ Import Summary"
echo "Imported:   ${#imported[@]}"
echo "Already in state: ${#already[@]}"
echo "Skipped:     ${#skipped[@]}"

if [[ ${#imported[@]} -gt 0 ]]; then
  printf "\n📦 Imported Resources:\n%s\n" "${imported[@]}"
fi
if [[ ${#skipped[@]} -gt 0 ]]; then
  printf "\n⚠️ Skipped (no ID):\n%s\n" "${skipped[@]}"
fi
