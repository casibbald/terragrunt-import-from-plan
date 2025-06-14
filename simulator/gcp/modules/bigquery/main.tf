// main.tf

resource "google_bigquery_dataset" "example" {
  dataset_id                 = local.dataset_id
  location                   = var.region
  project                    = var.project_id
  delete_contents_on_destroy = true
}

resource "google_bigquery_table" "example" {
  dataset_id = google_bigquery_dataset.example.dataset_id
  table_id   = local.table_id
  project    = var.project_id

  schema = jsonencode([
    {
      name = "name"
      type = "STRING"
      mode = "REQUIRED"
    },
    {
      name = "age"
      type = "INTEGER"
      mode = "NULLABLE"
    }
  ])
}

