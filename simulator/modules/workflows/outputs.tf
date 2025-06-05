// outputs.tf

output "workflow_name" {
  value       = google_workflows_workflow.example.name
  description = "Name of the created workflow"
}

