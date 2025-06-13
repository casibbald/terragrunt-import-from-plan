### ✅ **Already Implemented**
- [x] Simulated GCP infrastructure with reusable modules
- [x] Terragrunt environment in `envs/simulator/dev`
- [x] Generation of `tf.plan` binary
- [x] Conversion of `tf.plan` to JSON (`out.json`)
- [x] Module metadata JSON (`modules.json`)
- [x] Rust parsing logic for `out.json` and `modules.json`
- [x] Mapping of Terraform resources to module directories
- [x] Inference of Terraform resource IDs
- [x] Execution logic to run `terragrunt import` with dry-run toggle
- [x] Correct use of Terragrunt: various steps call 'terraform' when it should be 'terragrunt'
  - [x] Replace all `terraform` binary invocations with `terragrunt`
  - [x] Update `run_terraform_import` function to `run_terragrunt_import`
  - [x] Ensure `dry-run` uses `terragrunt import` command format
  - [x] Test error message paths for `terragrunt` missing binary edge case
  - [x] Confirm real-world compatibility with Terragrunt CLI behavior
- [x] Setup friendly bin name in Cargo.toml
- [x] Audit plan JSON for frequent alternative ID fields (`bucket`, `project`, `self_link`, etc.)
- [x] Update `main.rs` to deserialize plan JSON into `TerraformPlan` and extract `TerraformResource` list
- [x] Refactored `collect_resources()` into `utils::collect_all_resources()` to avoid code duplication

---

### 🛠️ **Remaining Tasks**

#### 🧪 Testing & Validation (High Priority)
- [ ] **Unit Tests & Code Quality**
  - [ ] Write unit tests for core functions:
    - [ ] `infer_resource_id()` with various resource types
    - [ ] `run_terragrunt_import()` with success/failure cases
    - [ ] `map_resources_to_modules()` with complex module structures
    - [ ] `extract_id_candidate_fields()` with different schemas
  - [ ] Add test coverage reporting
  - [ ] Implement integration tests with actual cloud providers
  - [ ] Create test fixtures for common resource types
  - [ ] Add performance benchmarks for large plans

#### 🔧 Provider Schema Extraction (High Priority)
- [ ] **Fix Provider Schema Extraction**
  - [ ] Debug and fix `write_provider_schema()` function
    - [ ] Verify correct working directory handling
    - [ ] Ensure proper error propagation
    - [ ] Add logging for debugging
  - [ ] Implement proper schema file writing
    - [ ] Handle file permissions correctly
    - [ ] Add file locking for concurrent access
    - [ ] Implement proper error handling for file operations
  - [ ] Add schema validation
    - [ ] Verify JSON structure
    - [ ] Check for required provider fields
    - [ ] Validate schema version compatibility
  - [ ] Add schema caching
    - [ ] Implement cache invalidation strategy
    - [ ] Add cache versioning
    - [ ] Handle cache corruption gracefully
  - [ ] Add CLI integration
    - [ ] Add `--force-schema-refresh` flag
    - [ ] Add schema path configuration
    - [ ] Add schema validation verbosity options

#### 🧩 Functionality Enhancements
- [ ] **Enhanced ID inference**: Use additional fields like `bucket`, `self_link`, etc.
  - [ ] Extract candidate fields dynamically from `provider_schemas.resource_schemas`
  - [ ] Score fields: prioritize common names like `id`, `name`, `bucket`, then fallback to others
  - [ ] Extend `infer_resource_id()` logic to rank and test those attributes per resource
  - [ ] Include verbose diagnostics to log which field was selected (if `--verbose` is set)
  - [ ] Write table-driven tests with various resource examples
  - [ ] Fail gracefully and provide helpful messages if no ID can be determined

- [ ] **External Provider Schema Integration**
  - [ ] Automatically run `terragrunt init` and then `terragrunt providers schema -json` inside `--module-root` to extract schema
  - [ ] Write schema to `.terragrunt-provider-schema.json`
  - [ ] Load this file dynamically as primary source for resource attribute scoring
  - [ ] Fallback to embedded plan schemas or heuristic scoring if missing
  - [ ] Skip CLI flag and integrate provider schema transparently for end users
  - [ ] Parse output of `terraform providers schema -json` into internal `ProviderSchemaMap`
  - [ ] Map resource types from plan to entries in schema file for ID logic
  - [ ] Validate schema contents and report any mismatches
  - [ ] Implement `write_provider_schema()` to invoke CLI and write file
  - [ ] Integrate invocation into `main.rs` pre-import step if schema file missing
  - [ ] Add CLI verbose logging around schema invocation and fallback path
  - [ ] Unit test for schema file generation using sandbox

- [ ] **Error Handling & Recovery**
  - [ ] Implement retry mechanisms for transient failures
  - [ ] Add detailed error reporting and logging
  - [ ] Create troubleshooting guide for common errors
  - [ ] Add error recovery strategies
  - [ ] Implement state file backup before imports
  - [ ] Add state file validation after imports
  - [ ] Add rollback capability for failed imports

- [ ] **Resource Management**
  - [ ] Support `--filter-type=TYPE` to import only specific resource types
  - [ ] Support `--address=ADDRESS` to import only a specific resource address
  - [ ] Handle non-importable resources gracefully (e.g., data sources)
  - [ ] Add parallel import execution option
  - [ ] Implement progress indicators for long-running imports

#### 📦 CLI & Config Polish
- [ ] **Input Validation & Security**
  - [ ] Add input validation for all CLI arguments
  - [ ] Implement secure credential handling
  - [ ] Add audit logging for import operations
  - [ ] Add security scanning in CI pipeline

- [ ] **Configuration & Paths**
  - [ ] Respect and validate `--module-root` directory structure
  - [ ] Ensure `--verbose` prints detailed diagnostics
  - [ ] Add support for configurable `--config-dir` path base
  - [ ] Ensure default paths function correctly within GitHub Actions
  - [ ] Implement configuration file support
  - [ ] Add interactive mode option

#### 🚀 GitHub Action Integration
- [ ] **Build & Release**
  - [ ] Compile Rust binary within GitHub Action
  - [ ] Run binary with simulated plan + module input
  - [ ] Emit structured import summary in GitHub job output
  - [ ] Set up automated release process
  - [ ] Implement version management
  - [ ] Add release notes generation
  - [ ] Create binary distribution pipeline

#### 📚 Developer Experience & Docs
- [ ] **Documentation**
  - [ ] Add architecture diagrams (sequence, context, flowcharts) to `docs/ARCH.md`
  - [ ] Document how schema extraction works in AGENTS.md or ARCH.md
  - [ ] Simplify onboarding with setup walkthrough and CLI examples
  - [ ] Add API documentation
  - [ ] Create detailed usage examples
  - [ ] Add contributing guidelines
  - [ ] Document best practices
  - [ ] Create comprehensive troubleshooting guide

#### 🔧 Performance & Optimization
- [ ] **Optimization Tasks**
  - [ ] Implement caching for provider schemas
  - [ ] Optimize memory usage for large plan processing
  - [ ] Add performance monitoring
  - [ ] Implement resource batching for large imports

---

### 📘 Educational Notes

- Use `if let Some(...)` instead of `match` when you only need to handle the `Some` case and optionally an `else`. It's cleaner and avoids extra nesting. For example:

```rust
// Preferred when handling one case:
if let Some(ref id) = inferred_id {
// use id
} else {
// handle absence
}

// More verbose equivalent:
match inferred_id {
Some(ref id) => { /* use id */ },
None => { /* handle absence */ },
}
```

This kind of pattern improves readability and aligns with idiomatic Rust.

More educational guidance will be added inline to future task breakdowns and code blocks where relevant.
