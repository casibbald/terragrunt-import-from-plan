// main.tf

resource "google_composer_environment" "example" {
  name    = local.env_name
  project = var.project_id
  region  = var.region

  config {
    node_count = 3

    software_config {
      image_version = "composer-2.0.33-airflow-2.3.4"
    }
  }
}

