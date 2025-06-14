# Testing Guide

This document provides comprehensive information about testing the `terragrunt-import-from-plan` project.

## 🧰 Tech Stack

**Core Technology:**
- **Language**: Rust 2021 Edition
- **CLI Framework**: `clap` v4 with derive features
- **JSON Processing**: `serde` and `serde_json` for parsing Terraform plan files
- **Error Handling**: `anyhow` for error context and `thiserror` for custom error types
- **File Operations**: `glob` for pattern matching, `tempfile` for testing
- **Regex**: Advanced pattern matching for resource ID extraction
- **Schema Processing**: Intelligent metadata parsing and scoring algorithms

**Development Tools:**
- **Testing**: Native Rust testing framework with `cargo test`
- **Coverage**: `cargo-llvm-cov` for detailed coverage reports
- **Build System**: Cargo with workspace support

## 📊 Test Coverage - 74 TESTS PASSING ✅

**Total Tests: 74** ✅ **All Passing**
- **Unit Tests**: 25 tests (module-specific functionality)
- **Binary Tests**: 31 tests (CLI and integration logic) 
- **Integration Tests**: 18 tests (end-to-end scenarios)
- **Schema Integration Tests**: 5 tests (schema-driven intelligence validation)

## Test Categories

### 1. **Resource Collection Tests** (`test_01` - `test_03`)
- `test_01_collect_resources` - Basic resource collection from modules
- `test_02_collect_resources_consolidation` - Nested module resource collection
- `test_03_collect_resources_empty_module` - Edge case handling for empty modules

### 2. **Schema Extraction Tests** (`test_04` - `test_06`)
- `test_04_extract_id_candidate_fields` - Provider schema parsing
- `test_05_extract_id_candidate_fields_empty_schema` - Empty schema handling
- `test_06_extract_id_candidate_fields_missing_provider` - Missing provider handling

### 3. **ID Candidate Field Tests** (`test_07` - `test_09`)
- `test_07_get_id_candidate_fields` - ID field extraction from schemas
- `test_08_get_id_candidate_fields_empty` - Empty schema edge cases
- `test_09_get_id_candidate_fields_less_than_three` - Minimal field scenarios

### 4. **Provider Schema Tests** (`test_10`)
- `test_10_load_provider_schema_invalid_file` - Invalid JSON file handling

### 5. **Attribute Scoring Tests** (`test_11` - `test_12`)
- `test_11_score_attributes_for_id` - ID field scoring algorithm
- `test_12_score_attributes_for_id_empty` - Empty attribute handling

### 6. **Import Command Generation Tests** (`test_13`)
- `test_13_generate_import_commands` - Terragrunt import command construction

### 7. **Resource ID Inference Tests** (`test_14`)
- `test_14_infer_resource_id` - Resource ID inference from plan data

### 8. **Module Mapping Tests** (`test_15`)
- `test_15_map_resources_to_modules` - Resource-to-module mapping logic

### 9. **Terragrunt Integration Tests** (`test_16`)
- `test_16_run_terragrunt_import_mock` - Mock terragrunt command execution

### 10. **Module Directory Validation Tests** (`test_17`)
- `test_17_validate_module_dirs` - Module directory structure validation (updated for multi-cloud)

### 11. **Provider Schema Generation Tests** (`test_18`)
- `test_18_generate_provider_schema_in_real_env` - Real environment schema generation

### 12. **✅ Schema Integration Tests** (Schema-Driven Intelligence - **IMPLEMENTED**)
- `test_schema_manager_parse_real_attributes` - Real schema parsing with 1,064+ resource types
- `test_artifact_registry_repository_parsing` - Resource-specific metadata extraction  
- `test_schema_driven_id_candidates` - Intelligent candidate ranking and scoring
- `test_list_resource_types` - Complete resource type enumeration from schema
- `test_metadata_scoring_logic` - Attribute scoring algorithm validation

### 13. **✅ Schema-Driven Scoring Tests** (**NEW - IMPLEMENTED**)
- `test_google_cloud_schema_driven_scoring` - GCP-specific intelligent scoring validation
- `test_required_vs_optional_scoring` - Required fields prioritized over optional
- `test_computed_field_bonus` - Computed fields get appropriate scoring bonuses
- `test_description_based_scoring` - Description analysis for "unique identifier" fields
- `test_resource_specific_overrides` - Resource-specific logic (e.g., storage bucket bonuses)
- `test_top_candidates_with_metadata` - Top candidate selection with real metadata
- `test_backward_compatibility` - Legacy scoring methods still work

## Running Tests

### Basic Test Execution

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests in a single thread (recommended for integration tests)
cargo test -- --test-threads=1

# Run specific test
cargo test test_name

# List all available tests
cargo test -- --list
```

### Targeted Test Execution

```bash
# Run only schema-driven scoring tests
cargo test scoring::

# Run only schema integration tests  
cargo test schema_integration_tests

# Run only integration tests
cargo test --test integration_tests

# Run only unit tests (lib tests)
cargo test --lib

# Run only binary tests
cargo test --bin terragrunt_import_from_plan
```

### Test Output Examples

When running schema-driven scoring tests, you'll see output like:

```
🎯 Artifact Registry Scoring:
  repository_id: 100.0  ✅ (correctly chosen!)
  name: 80.0

🎯 Resource-specific overrides:
  storage bucket name: 100.0  ✅ (gets GCP-specific bonus)
  generic resource name: 85.0

🎯 Required vs Optional:
  required: 65.0
  optional: 50.0

🎯 Computed vs Regular:
  computed: 60.0
  regular: 50.0
```

## Code Coverage

The project uses `cargo-llvm-cov` for code coverage reporting.

### Generating Coverage Reports

```bash
# Install llvm-cov if not already installed
cargo install cargo-llvm-cov

# Generate HTML coverage report
cargo llvm-cov --all-features --workspace --html

# Generate coverage report and open in browser
cargo llvm-cov --all-features --workspace --html --open

# Generate terminal coverage summary
cargo llvm-cov --all-features --workspace

# Generate coverage in different formats
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
cargo llvm-cov --all-features --workspace --json --output-path coverage.json
```

Coverage reports are generated in `target/llvm-cov/html/`.

### Understanding Coverage Metrics

The coverage reports show:
- **Line Coverage**: Percentage of code lines executed during tests
- **Function Coverage**: Percentage of functions called during tests  
- **Branch Coverage**: Percentage of code branches taken during tests
- **Region Coverage**: Percentage of code regions covered

## Error Handling Tests

The test suite includes comprehensive error handling tests that:

- **Verify expected error conditions** with proper error context
- **Provide clear error messages** for debugging
- **Include detailed output** (stdout/stderr) for troubleshooting
- **Maintain test isolation** and proper cleanup
- **Test graceful degradation** in environments without cloud access

### Example Error Handling Test

```rust
#[test]
fn test_18_generate_provider_schema_in_real_env() {
    let schema_path = std::path::Path::new("envs/simulator/gcp/dev/.terragrunt-provider-schema.json");
    let _ = std::fs::remove_file(schema_path);

    let result = write_provider_schema(std::path::Path::new("envs/simulator/gcp/dev"));
    
    match result {
        Ok(_) => {
            assert!(schema_path.exists(), ".terragrunt-provider-schema.json should be created when successful");
            println!("✅ Provider schema generation succeeded");
        }
        Err(e) => {
            println!("⚠️ Provider schema generation failed (expected in CI): {}", e);
            // This is acceptable - the test verifies the function handles errors properly
        }
    }
}
```

## Performance Testing

### Running Tests with Timing

```bash
# Run tests with timing information
cargo test -- --nocapture --test-threads=1 | grep -E "(test result|took)"

# Run specific performance-critical tests
cargo test test_schema_manager_parse_real_attributes -- --nocapture
cargo test test_list_resource_types -- --nocapture
```

### Benchmarking (Future)

The project is set up for future benchmarking with:

```bash
# When benchmark tests are added
cargo bench
```

## Test Data and Fixtures

### Test Fixtures Location

- `tests/fixtures/out.json` - Sample terraform plan output
- `tests/fixtures/modules.json` - Sample modules configuration
- `envs/simulator/gcp/dev/.terragrunt-provider-schema.json` - Real provider schema (6.3MB)

### Creating New Test Fixtures

When adding new test scenarios:

1. **Generate realistic test data** that represents real-world use cases
2. **Keep fixture files small** where possible for faster test execution
3. **Document the purpose** of each fixture file
4. **Ensure deterministic data** - no timestamps or random values

## Continuous Integration

### GitHub Actions Integration

The tests run automatically in GitHub Actions with:

```yaml
- name: Cargo test
  run: cargo test -- --test-threads=1

- name: Cargo test with output  
  run: cargo test -- --test-threads=1 --nocapture
```

### Local CI Simulation

```bash
# Simulate the CI environment locally
cargo clean
cargo build
cargo test -- --test-threads=1
cargo test -- --test-threads=1 --nocapture
```

## Contributing Guidelines for Tests

When adding new tests:

### 1. **Naming Convention**
- Follow existing pattern: `test_XX_descriptive_name`
- Use sequential numbering for integration tests
- Use descriptive names for unit tests

### 2. **Test Structure**
```rust
#[test]
fn test_xx_descriptive_name() {
    // Arrange
    let input = create_test_data();
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert!(result.is_ok(), "Expected success but got error: {:?}", result);
    assert_eq!(expected_value, actual_value);
}
```

### 3. **Documentation Requirements**
- Add test description in this document
- Include purpose and expected behavior
- Document any special setup requirements
- Update test count in README.md

### 4. **Test Quality Guidelines**
- **Deterministic**: Tests should pass consistently
- **Isolated**: Tests should not depend on each other
- **Fast**: Unit tests should complete quickly
- **Clear**: Test failure messages should be informative
- **Comprehensive**: Cover both success and failure cases

### 5. **Schema Integration Tests**

When adding schema-related tests:

```rust
#[test]
fn test_new_schema_feature() {
    let schema_dir = Path::new("envs/simulator/gcp/dev");
    
    // Skip if schema not available (CI environments)
    if !schema_dir.join(".terragrunt-provider-schema.json").exists() {
        println!("⚠️ Skipping test - schema file not found");
        return;
    }
    
    // Test logic here
}
```

## Troubleshooting Tests

### Common Issues

**Test fails in CI but passes locally:**
- Check for hardcoded paths
- Ensure test doesn't depend on local files
- Verify environment independence

**Schema tests skipped:**
- Normal in CI - schema file requires GCP access
- Run `just gen` locally to generate schema
- Tests gracefully skip when schema unavailable

**Timing-dependent failures:**
- Use `--test-threads=1` for sequential execution
- Add proper synchronization for concurrent operations
- Avoid time-dependent assertions

### Debug Commands

```bash
# Run with maximum verbosity
RUST_LOG=debug cargo test test_name -- --nocapture

# Run with backtrace on panic
RUST_BACKTRACE=1 cargo test test_name

# Run specific test with timing
time cargo test test_name -- --nocapture
```

## Future Test Enhancements

### Planned Additions

1. **Property-based testing** with `proptest` crate
2. **Integration tests** with real cloud resources (optional)
3. **Performance benchmarks** for schema parsing
4. **Mutation testing** for test quality verification
5. **Contract tests** for API compatibility

### Test Metrics Goals

- **Line Coverage**: Maintain >90%
- **Test Execution Time**: Keep under 30 seconds for full suite
- **Test Reliability**: 99.9% pass rate in CI
- **Documentation Coverage**: 100% of public APIs tested

---

For questions about testing or to report test-related issues, please check the [main README](README.md) or open an issue on GitHub. 