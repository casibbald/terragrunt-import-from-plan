### AGENTS.md

#### Version: v1.0.0

This document defines the operating principles and collaboration protocol for all agents (human or automated) contributing to this project.

---

### ✅ Verification Protocols

**No task shall be marked complete until:**
- [ ] The code is rendered in `markdown code blocks` (``` ```), and copied and tested locally.
- [ ] All compiler warnings or errors are reviewed and accepted.
- [ ] All new logic is tested with either:
    - [ ] Unit tests, or
    - [ ] Manual verification with CLI output pasted.
- [ ] Code is committed to Git.
- [ ] The branch is pushed to GitHub.
- [ ] The GitHub Action CI run passes (dry-run or live import).

Only once **all** of these are confirmed by a human operator, may the related task be marked `[x]` in `STACK_OF_TASKS.md`.

---

### 👷 Agent Operating Rules

- **Rendered Output:** All code must be shown in markdown blocks (```rust) for manual inspection.
- **No canvas-only updates.** All relevant changes must be reflected in terminal-compatible copy-paste blocks.
- **No speculative task completion.** Partial steps do not count as completion.
- **Tests First:** All new functionality requires TDD or post-validation before closing the loop.
- **Documentation-aware:** Update `STACK_OF_TASKS.md`, `AGENTS.md`, or related docs **only after user confirmation**.

---

### 🧭 Canonical Inputs

These flags must be supported and documented:
- `--plan`: Path to `out.json`
- `--modules`: Path to `modules.json`
- `--module-root`: Root directory where all modules live
- `--dry-run`: Flag to preview import commands
- `--verbose`: Detailed logs
- `--filter-type`: Limit to resource types
- `--address`: Limit to specific resource addresses

---

### 📂 File Structure Guardrails

- `main.rs`: Orchestration + CLI parsing
- `importer.rs`: Core logic for mapping, inference, importing
- `plan.rs`, `errors.rs`, `utils.rs`: Specialized modules if required
- `tests/fixtures/`: All plan/module test data

---

### 🔁 Revision Management

- Version each document (e.g., `v1.0.0`) at the top.
- Changelog maintained in commit history.
- Major refactors must be reflected in AGENTS and TASKS before merging.

---

This document is now enforced for all contributors and automation modules participating in the lifecycle of this importer.
