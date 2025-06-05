// main.tf

resource "google_artifact_registry_repository" "docker_repo" {
  project       = var.project_id
  repository_id = local.repo_id
  format        = "DOCKER"
  location      = var.region
  description   = "Simulated Docker repository"

  depends_on = [
    var.api_enablement_dependency
  ]
}