// outputs.tf

output "service_account_email" {
  description = "The email of the service account"
  value       = google_service_account.example_sa.email
}

output "custom_role_name" {
  description = "The full name of the custom IAM role"
  value       = google_project_iam_custom_role.developer.name
}

