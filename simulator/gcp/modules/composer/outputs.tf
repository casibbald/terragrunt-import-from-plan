// outputs.tf

output "composer_env_name" {
  value       = google_composer_environment.example.name
  description = "Name of the Composer environment"
}

