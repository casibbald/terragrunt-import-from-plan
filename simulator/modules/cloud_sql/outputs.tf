// outputs.tf

output "instance_connection_name" {
  value       = google_sql_database_instance.example.connection_name
  description = "Connection name of the SQL instance"
}

output "database_name" {
  value       = google_sql_database.default.name
  description = "Name of the SQL database"
}

output "user_name" {
  value       = google_sql_user.default.name
  description = "SQL user name"
}

