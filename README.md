# Terragrunt Import From Plan

[![Test](https://github.com/casibbald/terragrunt-import-from-plan/actions/workflows/test.yml/badge.svg)](https://github.com/casibbald/terragrunt-import-from-plan/actions/workflows/test.yml)
[![Release](https://github.com/casibbald/terragrunt-import-from-plan/actions/workflows/release.yml/badge.svg?branch=main)](https://github.com/casibbald/terragrunt-import-from-plan/actions/workflows/release.yml)

### 🤔 Why Use This Action?

When running `terragrunt plan`, you may have resources marked for creation but not yet in the Terraform state. This action
helps you automatically import those resources, ensuring your state file is up-to-date without manual intervention. 

This is particularly useful for CI/CD pipelines where you want to ensure all resources are managed correctly without having
to run `terraform import` commands manually. 

It is beneficial in corporate settings where environments use restrictive GCP, AWS, or Azure IAM, requiring complex service 
account impersonation. Often, direct access to import resources is not always possible in the local environment, or is tedious 
to set up, or there are security restrictions from having local access to these environments, especially in Production settings. 

This resource ensures your Terraform state remains consistent with the infrastructure, reducing the risk of drift mid-deployment.

### 🚀 Features

- 🔍 Parses `terraform show -json` output from a Terraform plan
- 📦 Dynamically identifies resources with `create`, `create+update`, or `replace` actions
- 🔑 Extracts importable IDs from fields like `repository_id`, `name`, `bucket`, `id`, `arn`, or full Azure resource IDs
- 🛠 Supports optional cloud-specific ID formatting:
  - **GCP**: prefix `projects/$PROJECT_ID/locations/$LOCATION/repositories/...`
  - **AWS**: uses raw `arn:` strings when detected (e.g., `arn:aws:iam::...:role/...`)
  - **Azure**: uses full `/subscriptions/...` IDs directly
- 📊 Outputs a summary of:
  - Imported resources
  - Already in state
  - Skipped (due to missing ID)

### 🧪 Included Test Suite

Run `./test/entrypoint_test.sh` to validate logic with mocked input — no infrastructure required.

---

## 🛠 Usage

```yaml
jobs:
  import-plan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Terragrunt
        uses: metro-digital/cf-github-actions/terragrunt-setup@v1.0.0

      - name: Terragrunt Plan
        run: terragrunt plan -out=${{ github.sha }}.plan
        working-directory: ${{ github.workspace }}/${{ matrix.environment }}/some-module

      - name: Import resources from plan
        uses: casibbald/terragrunt-import-from-plan@v1.0.0
        with:
          working-directory: ${{ github.workspace }}/${{ matrix.environment }}/some-module
          plan-file: ${{ github.sha }}.plan
```

### 🌍 Cloud-Specific Settings

#### For GCP:

Optionally set `PROJECT_ID` and `LOCATION` as environment variables for GCP-style imports:

```yaml
env:
  PROJECT_ID: your-project-id
  LOCATION: your-location
```

#### For AWS or Azure:
If your resources use `arn:` or `/subscriptions/...`, no additional variables are needed — the action detects and imports using them directly.

---

## 📄 Output Example

```
🔍 Checking google_artifact_registry_repository.remote_repos["mock-repo"]...
📦 Importing google_artifact_registry_repository.remote_repos["mock-repo"] with ID: projects/my-project/locations/europe-west1/repositories/mock-repo

✅ Import Summary
Imported:   1
Already in state: 0
Skipped:     0

📦 Imported Resources:
google_artifact_registry_repository.remote_repos["mock-repo"]
```

### 📄 Example Plan JSON Snippets

#### GCP Artifact Registry
```json
{
  "address": "google_artifact_registry_repository.remote[\"foo\"]",
  "change": {
    "actions": ["create"],
    "after": {
      "repository_id": "foo"
    }
  }
}
```

#### AWS IAM Role
```json
{
  "address": "aws_iam_role.role[\"app\"]",
  "change": {
    "actions": ["create"],
    "after": {
      "arn": "arn:aws:iam::123456789012:role/app"
    }
  }
}
```

#### Azure Storage Account
```json
{
  "address": "azurerm_storage_account.example[\"main\"]",
  "change": {
    "actions": ["create"],
    "after": {
      "id": "/subscriptions/0000-1111-2222-3333/resourceGroups/rg/providers/Microsoft.Storage/storageAccounts/myaccount"
    }
  }
}
```

## Testing

### Test Stack

The project uses a comprehensive test suite with the following components:

1. **Integration Tests** (`tests/integration_tests.rs`)
   - 27 tests covering all major functionality
   - Tests are ordered and named with prefixes (e.g., `test_01_`, `test_02_`) for clear execution order
   - Includes tests for error cases with clear `[EXPECTED ERROR]` output

2. **Unit Tests** (in respective module files)
   - Tests for individual components and functions
   - Located alongside the code they test

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests in a single thread (recommended for integration tests)
cargo test -- --test-threads=1

# Run specific test
cargo test test_name
```

### Code Coverage

The project uses `cargo-llvm-cov` for code coverage reporting. To generate coverage reports:

```bash
# Generate HTML coverage report
cargo llvm-cov --all-features --workspace --html

# Generate coverage report and open in browser
cargo llvm-cov --all-features --workspace --html --open
```

Coverage reports are generated in `target/llvm-cov/html/`.

### Test Categories

1. **Resource Collection Tests**
   - `test_03_collect_resources`
   - `test_04_collect_all_resources`
   - `test_05_collect_resources_empty_module`

2. **Schema Tests**
   - `test_06_extract_id_candidate_fields`
   - `test_07_extract_id_candidate_fields_empty_schema`
   - `test_08_extract_id_candidate_fields_missing_provider`

3. **Provider Schema Tests**
   - `test_09_write_provider_schema`
   - `test_10_write_provider_schema_invalid_dir`
   - `test_11_write_provider_schema_readonly_dir`
   - `test_12_write_provider_schema_basic`
   - `test_13_write_provider_schema_invalid_output`
   - `test_14_write_provider_schema_terragrunt_not_found`

4. **ID Candidate Tests**
   - `test_15_get_id_candidate_fields`
   - `test_16_get_id_candidate_fields_empty`
   - `test_17_get_id_candidate_fields_less_than_three`

5. **Provider Schema Loading Tests**
   - `test_18_load_provider_schema`
   - `test_19_load_provider_schema_invalid_file`
   - `test_20_load_provider_schema_invalid_json`

6. **Attribute Scoring Tests**
   - `test_21_score_attributes_for_id`
   - `test_22_score_attributes_for_id_empty`

7. **Import Command Tests**
   - `test_23_generate_import_commands`
   - `test_24_infer_resource_id`
   - `test_25_map_resources_to_modules`

8. **Terragrunt Integration Tests**
   - `test_26_run_terragrunt_import_mock`
   - `test_27_validate_module_dirs`

### Error Handling

The test suite includes comprehensive error handling tests that:
- Verify expected error conditions
- Provide clear error messages with `[EXPECTED ERROR]` prefix
- Include detailed output (stdout/stderr) for debugging
- Maintain test isolation and cleanup

## Contributing

When adding new tests:
1. Follow the existing naming convention (`test_XX_`)
2. Include error handling tests where appropriate
3. Add clear error messages for expected failures
4. Update this documentation with any new test categories

# Contributing

## Rust version

### 🧪 Run Tests

```bash
cargo test
```

### Run Locally

```bash
 cargo run -- --plan tests/fixtures/out.json --modules tests/fixtures/modules.json --module-root simulator/modules --dry-run

```



## Legacy Bash version

### 🧪 Run Tests

```bash
./test/entrypoint_test.sh
```

This will run a mocked import against a fake `plan.json` and show the correct import logic.

---

## 🛡 License
[MIT](LICENSE)

## Coverage Reports

To generate the LLVM coverage report, run:

```bash
cargo llvm-cov --html
```

The coverage report is saved to `target/llvm-cov/html`.
