### ✅ **Already Implemented**
- [x] Simulated GCP infrastructure with reusable modules
- [x] Terragrunt environment in `envs/simulator/dev`
- [x] Generation of `tf.plan` binary
- [x] Conversion of `tf.plan` to JSON (`out.json`)
- [x] Module metadata JSON (`modules.json`)
- [x] Rust parsing logic for `out.json` and `modules.json`
- [x] Mapping of Terraform resources to module directories
- [x] Inference of Terraform resource IDs

---

### 🔄 **In Progress: Execution logic to run `terraform import` with dry-run toggle**
- [ ] CLI Argument Handling for `--dry-run`
- [ ] Iteration Over Inferred Resources for Execution
- [ ] Conditional Branching: Print vs Execute Command
- [ ] Command Construction and Path Handling
- [ ] Error Handling and Logging for Execution Failures
- [ ] Ensure Base Path (e.g., `simulator/`) Prefixes `config-dir`
- [ ] Display Executed Commands for Transparency in Dry-Run

---

### 🛠️ **Remaining Tasks**

#### 🧩 Functionality Enhancements
- [ ] **Support filter by resource type** via `--filter-type`
- [ ] **Support specific resource address** imports (e.g., `--address module.foo.google_x.bar`)
- [ ] **Enhanced ID inference**: use more sophisticated heuristics when `name` and `id` are not sufficient
- [ ] **Handle non-importable resources gracefully**

#### 📦 CLI & Config Polish
- [ ] **Ensure `--verbose` prints diagnostics**
- [ ] **Add support for configurable `--config-dir` path base**
- [ ] **Default paths work cleanly from GitHub Action (not just local)**

#### 🧪 Testing & Validation
- [ ] **Unit tests for all major functionality**
- [ ] **Integration test**: Generate real `tf.plan` via Terragrunt and validate import end-to-end
- [ ] **Mocked `terraform import` testing** for test mode without GCP credentials
- [ ] **CI GitHub Action test** with dry-run verification

#### 🚀 GitHub Action Integration
- [ ] **Build and compile the Rust binary** within the GitHub Action
- [ ] **Run with simulated plan + modules JSON**
- [ ] **Emit structured output for GitHub summary log**
