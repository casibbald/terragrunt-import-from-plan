// main.tf

resource "google_workflows_workflow" "example" {
  name     = local.workflow_name
  project  = var.project_id
  region   = var.region

  description = "Simulated workflow"

  source_contents = <<EOF
main:
  steps:
  - init:
      assign:
      - result: "Hello from Workflows"
  - return:
      return: $${result}
EOF
}

