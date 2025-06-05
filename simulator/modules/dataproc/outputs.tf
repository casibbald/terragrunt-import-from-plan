// outputs.tf

output "dataproc_cluster_name" {
  value       = google_dataproc_cluster.example.name
  description = "Name of the Dataproc cluster"
}

