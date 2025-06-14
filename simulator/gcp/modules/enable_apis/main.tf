// main.tf

resource "google_project_service" "required_services" {
  for_each = toset(local.required_services)
  project  = var.project_id
  service  = each.key
  disable_on_destroy = false
}

