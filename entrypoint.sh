#!/bin/bash
set -euo pipefail

PLAN_FILE=${1:-tf.plan}

# Use existing JSON plan file if provided
if [[ "$PLAN_FILE" == *.json ]]; then
  if [[ "$PLAN_FILE" != "plan.json" ]]; then
    cp "$PLAN_FILE" plan.json
  fi
else
  if [[ ! -f "$PLAN_FILE" ]]; then
    echo "❌ Error: $PLAN_FILE not found. Run 'terragrunt plan -out=$PLAN_FILE' before this action."
    exit 1
  fi
  # Export plan to JSON
  terraform show -json "$PLAN_FILE" > plan.json
fi

imported=()
skipped=()
already=()

# Process each creatable resource
while read -r line; do
  address=$(echo "$line" | jq -r '.address')
  after_obj=$(echo "$line" | jq '.after')

  id=$(echo "$after_obj" | jq -r '
    if has("arn") then .arn
    elif has("id") and (.id | test("^/subscriptions/")) then .id
    elif has("name") then .name
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
  if [[ -n "${PROJECT_ID:-}" && -n "${LOCATION:-}" && "$id" != "arn:"* && "$id" != "/subscriptions/"* ]]; then
    id="projects/${PROJECT_ID}/locations/${LOCATION}/repositories/${id}"
  fi

  echo "🔍 Checking $address..."
  if terragrunt state show "$address" > /dev/null 2>&1; then
    echo " ✅ $address already in state"
    already+=("$address")
  else
    echo " 📦 Importing $address with ID: $id"
    terragrunt import "$address" "$id"
    imported+=("$address")
  fi

done < <(jq -c '
  .resource_changes[]
  | select(.change.actions | inside(["create"]) or inside(["create", "update"]) or inside(["create", "delete", "update"]))
  | {address: .address, after: .change.after}
' plan.json)

# Summary
echo " ✅ Import Summary"
echo "Imported:   ${#imported[@]}"
echo "Already in state: ${#already[@]}"
echo "Skipped:     ${#skipped[@]}"

if [[ ${#imported[@]} -gt 0 ]]; then
  printf " 📦 Imported Resources:\n%s\n" "${imported[@]}"
fi
if [[ ${#skipped[@]} -gt 0 ]]; then
  printf " ⚠️ Skipped (no ID):\n%s\n" "${skipped[@]}"
fi
