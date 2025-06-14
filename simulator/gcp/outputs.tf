# Project Information
output "project_id" {
  description = "The Google Cloud project ID"
  value       = var.project_id
}

output "region" {
  description = "The Google Cloud region"
  value       = var.region
}

# Networking Outputs
output "vpc_name" {
  description = "The name of the VPC network"
  value       = module.networking.vpc_name
  depends_on  = [module.networking]
}

output "subnet_name" {
  description = "The name of the subnet"
  value       = module.networking.subnet_name
  depends_on  = [module.networking]
}

output "static_ip" {
  description = "The reserved static IP address"
  value       = module.networking.static_ip
  depends_on  = [module.networking]
}

# Storage Outputs
output "storage_bucket_name" {
  description = "The name of the Cloud Storage bucket"
  value       = module.storage.bucket_name
  depends_on  = [module.storage]
}

# IAM Outputs
output "service_account_email" {
  description = "The email of the created service account"
  value       = module.iam.service_account_email
  depends_on  = [module.iam]
}

output "custom_role_name" {
  description = "The full name of the custom IAM role"
  value       = module.iam.custom_role_name
  depends_on  = [module.iam]
}

# BigQuery Outputs
output "bigquery_dataset_id" {
  description = "The ID of the BigQuery dataset"
  value       = module.bigquery.dataset_id
  depends_on  = [module.bigquery]
}

output "bigquery_table_id" {
  description = "The ID of the BigQuery table"
  value       = module.bigquery.table_id
  depends_on  = [module.bigquery]
}

# GKE Outputs
output "gke_cluster_name" {
  description = "The name of the GKE cluster"
  value       = module.gke.cluster_name
  depends_on  = [module.gke]
}

output "gke_node_pool_name" {
  description = "The name of the GKE node pool"
  value       = module.gke.node_pool_name
  depends_on  = [module.gke]
} 