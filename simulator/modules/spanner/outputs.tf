// outputs.tf

output "spanner_instance" {
  value       = google_spanner_instance.example.name
  description = "Name of the Spanner instance"
}

output "spanner_database" {
  value       = google_spanner_database.example.name
  description = "Name of the Spanner database"
}
