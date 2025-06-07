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

#### 🧩 Functionality Enhancements
- [ ] **Enhanced ID inference**: Use additional fields like `bucket`, `self_link`, etc.
  - [ ] Extract candidate fields dynamically from `provider_schemas.resource_schemas`
  - [ ] Score fields: prioritize common names like `id`, `name`, `bucket`, then fallback to others
  - [ ] Extend `infer_resource_id()` logic to rank and test those attributes per resource
  - [ ] Include verbose diagnostics to log which field was selected (if `--verbose` is set)
  - [ ] Write table-driven tests with various resource examples
  - [ ] Fail gracefully and provide helpful messages if no ID can be determined
- [ ] **Support `--filter-type=TYPE`** to import only specific resource types
- [ ] **Support `--address=ADDRESS`** to import only a specific resource address
- [ ] **Handle non-importable resources gracefully (e.g., data sources)**

#### 📦 CLI & Config Polish
- [ ] **Respect and validate `--module-root` directory structure**
- [ ] **Ensure `--verbose` prints detailed diagnostics**
- [ ] **Add support for configurable `--config-dir` path base**
- [ ] **Ensure default paths function correctly within GitHub Actions**

#### 🧪 Testing & Validation
- [ ] **Unit tests** for all major functions (`infer_resource_id`, `run_terragrunt_import`, etc.)
- [ ] **Integration test**: Generate real `tf.plan` via Terragrunt and validate imports
- [ ] **Mocked import testing** to simulate imports without actual GCP
- [ ] **CI GitHub Action test** validating dry-run output

#### 🚀 GitHub Action Integration
- [ ] **Compile Rust binary within GitHub Action**
- [ ] **Run binary with simulated plan + module input**
- [ ] **Emit structured import summary in GitHub job output**

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
