# Terragrunt Import From Plan

This GitHub Action dynamically imports resources detected as "to be created" in a `terragrunt plan` output into the Terraform state.

It ensures that resources which already exist in the cloud but are not yet tracked by Terraform are brought into state automatically before apply.

---

## 📦 Features

- Runs `terragrunt plan -out=tf.plan`
- Parses the JSON plan for any resources marked for `create`
- For each resource:
    - Runs `terragrunt state show`
    - Runs `terragrunt import` only if the resource is not already in state

---

## 🔧 Inputs

| Name                | Description                                          | Default |
|---------------------|------------------------------------------------------|---------|
| `working-directory` | Path to the Terragrunt configuration to execute from | `.`     |

---

## 🚀 Usage

```yaml
jobs:
  terragrunt-import:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Terragrunt
        uses: metro-digital/cf-github-actions/terragrunt-setup@v1

      - name: Run Terragrunt Import From Plan
        uses: casibbald/terragrunt-import-from-plan@main
        with:
          working-directory: ./envs/dev/registry
```

---

## 📝 Requirements

- `terragrunt` and `terraform` must be installed in the runner (use a setup action like `metro-digital/cf-github-actions/terragrunt-setup@v1`).
- This action assumes `terragrunt plan -out=tf.plan` is valid in the target directory.

---

## 🧪 Example Plan Result

Given this in `plan.json`:
```json
{
  "resource_changes": [
    {
      "address": "google_artifact_registry_repository.remote_repos[\"foo\"]",
      "change": {
        "actions": ["create"],
        "after": {
          "repository_id": "foo"
        }
      }
    }
  ]
}
```
This action will run:
```bash
terragrunt import google_artifact_registry_repository.remote_repos["foo"] projects/your-project/locations/your-region/repositories/foo
```

---

## 🤝 Contributing
Pull requests and feedback welcome!

---

## 🛡 License
[MIT](LICENSE)
