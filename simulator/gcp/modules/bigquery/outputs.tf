// outputs.tf

output "dataset_id" {
  value       = google_bigquery_dataset.example.dataset_id
  description = "The ID of the BigQuery dataset"
}

output "table_id" {
  value       = google_bigquery_table.example.table_id
  description = "The ID of the BigQuery table"
}

