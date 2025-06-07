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

---

### 🛠️ **Remaining Tasks**

#### 🧩 Functionality Enhancements
- [ ] **Correct use of Terragrunt**: various steps call 'terraform' when it should be 'terragrunt'
    - [x] Replace all `terraform` binary invocations with `terragrunt`
    - [x] Update `run_terraform_import` function to `run_terragrunt_import`
    - [x] Ensure `dry-run` uses `terragrunt import` command format
    - [ ] Test error message paths for `terragrunt` missing binary edge case
    - [ ] Confirm real-world compatibility with Terragrunt CLI behavior
- [ ] **Enhanced ID inference**: Use additional fields like `bucket`, `self_link`, etc.
    - [ ] Audit plan JSON for frequent alternative ID fields (`bucket`, `project`, `self_link`, etc.)
    - [ ] Extend `infer_resource_id()` logic to:
        - [ ] Try `id`
        - [ ] Fallback to `name`
        - [ ] Try type-specific fields like `bucket`, `repository_id`, etc.
    - [ ] Refactor `infer_resource_id()` to include verbose logging for each attempt
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
