// outputs.tf

output "enable_apis" {
  value       = [for s in google_project_service.required_services : s.service]
  description = "List of enabled APIs in the project"
}