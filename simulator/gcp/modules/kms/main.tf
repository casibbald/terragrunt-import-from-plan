// main.tf

resource "google_kms_key_ring" "example" {
  name     = local.key_ring_name
  location = var.location
  project  = var.project_id
}

resource "google_kms_crypto_key" "example" {
  name            = local.crypto_key_name
  key_ring        = google_kms_key_ring.example.id
  rotation_period = "100000s"
}

