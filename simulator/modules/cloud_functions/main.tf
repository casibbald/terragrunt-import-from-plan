// main.tf

resource "google_storage_bucket" "example" {
  name     = local.bucket_name
  location = var.region
  force_destroy = true

  versioning {
    enabled = true
  }
}

resource "google_cloudfunctions_function" "example" {
  name        = local.function_name
  description = "Example Cloud Function"
  runtime     = "nodejs18"
  region      = var.region
  project     = var.project_id

  entry_point = "helloWorld"

  source_archive_bucket = google_storage_bucket.example.name
  source_archive_object = google_storage_bucket_object.source_archive.name

  trigger_http = true
  available_memory_mb = 128
}

resource "google_storage_bucket" "source" {
  name     = "cloud-functions-source-${var.project_id}"
  location = var.region
  project  = var.project_id
}

resource "google_storage_bucket_object" "source_archive" {
  name   = "source.zip"
  bucket = google_storage_bucket.source.name
  source = "../function-source/source.zip" # Assume local archive exists for sim
}