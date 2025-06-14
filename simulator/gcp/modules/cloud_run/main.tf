// main.tf

resource "google_cloud_run_service" "example" {
  name     = local.service_name
  location = var.region
  project  = var.project_id

  template {
    spec {
      containers {
        image = "gcr.io/cloudrun/hello"
      }
    }
  }

  traffic {
    percent         = 100
    latest_revision = true
  }
}

resource "google_cloud_run_service_iam_member" "invoker" {
  location = google_cloud_run_service.example.location
  project  = var.project_id
  service  = google_cloud_run_service.example.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}

