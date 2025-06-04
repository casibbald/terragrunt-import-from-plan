# Terragrunt Import From Plan

[![Test](https://github.com/casibbald/terragrunt-import-from-plan/actions/workflows/test.yml/badge.svg)](https://github.com/casibbald/terragrunt-import-from-plan/actions/workflows/test.yml)
[![Release](https://github.com/casibbald/terragrunt-import-from-plan/actions/workflows/release.yml/badge.svg)](https://github.com/casibbald/terragrunt-import-from-plan/actions/workflows/release.yml)

Automatically import Terraform resources from a Terragrunt plan.

This GitHub Action analyzes the output of `terragrunt plan`, identifies resources marked for creation, and attempts to import them into the Terraform state — skipping those already managed.

---

## 🚀 Features

- Parses `terraform show -json tf.plan`
- Supports actions: `create`, `create+update`, and `replace`
- Dynamically extracts IDs from fields like `name`, `id`, `repository_id`, `bucket`
- Optional `PROJECT_ID` and `LOCATION` env vars to build GCP-style IDs
- Prints a summary of imported, already-managed, and skipped resources

---

## 🛠 Usage

```yaml
jobs:
  import-plan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Terragrunt
        uses: metro-digital/cf-github-actions/terragrunt-setup@v1

      - name: Terragrunt Plan
        run: terragrunt plan -out=tf.plan
        working-directory: ./envs/dev/registry

      - name: Import resources from plan
        uses: casibbald/terragrunt-import-from-plan@v1
        with:
          working-directory: ./envs/dev/registry
```

Optionally set `PROJECT_ID` and `LOCATION` as environment variables for GCP-style imports:

```yaml
env:
  PROJECT_ID: cf-sam-ci-d0
  LOCATION: europe-west1
```

---

## 📄 Output Example

```
🔍 Checking google_artifact_registry_repository.remote_repos["mock-repo"]...
📦 Importing google_artifact_registry_repository.remote_repos["mock-repo"] with ID: projects/cf-sam-ci-d0/locations/europe-west1/repositories/mock-repo

✅ Import Summary
Imported:   1
Already in state: 0
Skipped:     0

📦 Imported Resources:
google_artifact_registry_repository.remote_repos["mock-repo"]
```

---

## 🧪 Run Tests

```bash
./test/entrypoint_test.sh
```

This will run a mocked import against a fake `plan.json` and show the correct import logic.

---

## 🛡 License
[MIT](LICENSE)
