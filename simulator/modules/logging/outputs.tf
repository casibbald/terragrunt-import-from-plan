// outputs.tf

output "log_sink_name" {
  value       = google_logging_project_sink.example.name
  description = "Name of the created log sink"
}

output "log_metric_name" {
  value       = google_logging_metric.example.name
  description = "Name of the log-based metric"
}
