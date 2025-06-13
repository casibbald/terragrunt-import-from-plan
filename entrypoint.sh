#!/bin/bash
set -euo pipefail

PLAN_FILE=${1:-tf.plan}
MODULES_FILE=${2:-modules.json}
DRY_RUN=${DRY_RUN:-false}
VERBOSE=${VERBOSE:-false}
WORKING_DIR=${WORKING_DIR:-"."}

echo "🚀 Terragrunt Import from Plan (Rust Edition)"
echo "📁 Plan file: $PLAN_FILE"
echo "📋 Modules file: $MODULES_FILE" 
echo "🏠 Working directory: $WORKING_DIR"

# Convert plan file to JSON if needed
if [[ "$PLAN_FILE" == *.json ]]; then
  PLAN_JSON="$PLAN_FILE"
else
  if [[ ! -f "$PLAN_FILE" ]]; then
    echo "❌ Error: $PLAN_FILE not found. Run 'terragrunt plan -out=$PLAN_FILE' before this action."
    exit 1
  fi
  echo "🔄 Converting plan to JSON..."
  PLAN_JSON="plan.json"
  terraform show -json "$PLAN_FILE" > "$PLAN_JSON"
fi

# Generate modules.json if it doesn't exist
if [[ ! -f "$MODULES_FILE" ]]; then
  echo "📝 Generating modules.json..."
  terragrunt graph-dependencies --terragrunt-modules-that-include terragrunt.hcl --terragrunt-json > "$MODULES_FILE"
fi

# Build the Rust binary if needed
if [[ ! -f "./target/release/terragrunt_import_from_plan" ]]; then
  echo "🔨 Building Rust binary..."
  cargo build --release
fi

# Prepare command arguments
ARGS=(
  --plan "$PLAN_JSON"
  --modules "$MODULES_FILE"
  --working-directory "$WORKING_DIR"
)

if [[ "$DRY_RUN" == "true" ]]; then
  ARGS+=(--dry-run)
fi

if [[ "$VERBOSE" == "true" ]]; then
  ARGS+=(--verbose)
fi

# Execute the Rust-based import tool
echo "⚡ Running terragrunt import..."
./target/release/terragrunt_import_from_plan "${ARGS[@]}"

echo "✅ Import process completed!"
