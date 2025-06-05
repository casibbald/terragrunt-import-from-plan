// main.tf

resource "google_secret_manager_secret" "example" {
  secret_id = "secret"
  project   = var.project_id
  labels = {
    label = "my-label"
  }

  replication {
    user_managed {
      replicas {
        location = "us-central1"
      }
      replicas {
        location = "us-east1"
      }
    }
  }
}

resource "google_secret_manager_secret_version" "version" {
  secret      = google_secret_manager_secret.example.id
  secret_data = var.secret_value
}



