// outputs.tf

output "bucket_name" {
  value       = google_storage_bucket.example.name
  description = "The name of the created GCS bucket"
}

