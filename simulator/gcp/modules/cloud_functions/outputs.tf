// outputs.tf

output "function_url" {
  value       = google_cloudfunctions_function.example.https_trigger_url
  description = "Cloud Function trigger URL"
}