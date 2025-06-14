// outputs.tf

output "cluster_name" {
  value       = google_container_cluster.primary.name
  description = "Name of the GKE cluster"
}

output "node_pool_name" {
  value       = google_container_node_pool.primary_nodes.name
  description = "Name of the GKE node pool"
}

