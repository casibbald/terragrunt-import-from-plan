// outputs.tf

output "repository_id" {
  value       = google_artifact_registry_repository.docker_repo.repository_id
  description = "The ID of the created Artifact Registry repository"
}
