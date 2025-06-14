// main.tf

resource "google_service_account" "example_sa" {
  account_id   = "example-sa"
  display_name = "Example Service Account"
  project      = var.project_id
}

resource "google_service_account_key" "example_sa_key" {
  service_account_id = google_service_account.example_sa.name
}

resource "google_project_iam_custom_role" "developer" {
  role_id     = "customDeveloperRole"
  title       = "Custom Developer"
  description = "A custom role for developers"
  permissions = [
    "compute.instances.get",
    "resourcemanager.projects.get"
  ]
  project = var.project_id
}

resource "google_project_iam_binding" "developer_binding" {
  project = var.project_id
  role    = google_project_iam_custom_role.developer.name
  members = [
    "serviceAccount:${google_service_account.example_sa.email}"
  ]
}

resource "google_service_account_iam_binding" "sa_binding" {
  service_account_id = google_service_account.example_sa.name
  role               = "roles/iam.serviceAccountTokenCreator"
  members = [
    "serviceAccount:${google_service_account.example_sa.email}"
  ]
}

resource "google_project_iam_member" "project_owner" {
  project = var.project_id
  role    = "roles/owner"
  member  = "serviceAccount:${google_service_account.example_sa.email}"
}

