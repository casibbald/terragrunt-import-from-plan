// main.tf

resource "google_container_cluster" "primary" {
  name     = local.cluster_name
  location = var.region
  project  = var.project_id

  remove_default_node_pool = true
  initial_node_count       = 1

  ip_allocation_policy {}
}

resource "google_container_node_pool" "primary_nodes" {
  name     = local.node_pool_name
  location = var.region
  cluster  = google_container_cluster.primary.name
  project  = var.project_id

  node_config {
    machine_type = "e2-medium"
    oauth_scopes = [
      "https://www.googleapis.com/auth/cloud-platform"
    ]
  }

  initial_node_count = 1
}

