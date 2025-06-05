// outputs.tf

output "secret_name" {
  value       = google_secret_manager_secret.example.name
  description = "The name of the created secret"
}

output "secret_version" {
  value       = google_secret_manager_secret_version.version.name
  description = "Version ID of the created secret"
}
