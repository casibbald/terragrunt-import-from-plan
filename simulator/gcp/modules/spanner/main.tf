// main.tf

resource "google_spanner_instance" "example" {
  name         = local.instance_name
  config       = "regional-${var.region}"
  display_name = "Simulated Spanner Instance"
  num_nodes    = 1
  project      = var.project_id
}

resource "google_spanner_database" "example" {
  name     = local.database_name
  instance = google_spanner_instance.example.name
  project  = var.project_id

  ddl = [
    "CREATE TABLE users (id STRING(36) NOT NULL, name STRING(1024)) PRIMARY KEY(id)"
  ]
}


