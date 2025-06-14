// main.tf

resource "google_dataproc_cluster" "example" {
  name    = local.cluster_name
  region  = var.region
  project = var.project_id

  cluster_config {
    master_config {
      num_instances = 1
      machine_type  = "n1-standard-1"
    }

    worker_config {
      num_instances = 2
      machine_type  = "n1-standard-1"
    }
  }
}

