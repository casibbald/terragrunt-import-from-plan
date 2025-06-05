// main.tf

resource "google_sql_database_instance" "example" {
  name             = local.instance_name
  database_version = "POSTGRES_14"
  region           = var.region
  project          = var.project_id

  settings {
    tier = "db-f1-micro"
  }
}

resource "google_sql_database" "default" {
  name     = local.db_name
  instance = google_sql_database_instance.example.name
  project  = var.project_id
}

resource "google_sql_user" "default" {
  name     = local.user_name
  instance = google_sql_database_instance.example.name
  password = "supersecure"
  project  = var.project_id
}

