// main.tf

resource "google_storage_bucket" "example" {
  name          = local.bucket_name
  location      = var.region
  force_destroy = true

  versioning {
    enabled = true
  }
}

resource "google_storage_bucket_iam_binding" "binding" {
  bucket = google_storage_bucket.example.name
  role   = "roles/storage.objectViewer"
  members = [
    "allUsers"
  ]
}

# Placeholder for a future Transfer Job
resource "google_storage_transfer_job" "placeholder" {
  project     = var.project_id
  description = "Placeholder transfer job for simulator"
  transfer_spec {
    # Example minimal transfer_spec; adjust as needed for your use case
    gcs_data_source {
      bucket_name = "source-bucket"
    }
    gcs_data_sink {
      bucket_name = google_storage_bucket.example.name
    }
  }
  lifecycle {
    prevent_destroy = true
  }
}
