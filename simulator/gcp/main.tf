# This file includes all GCP module declarations, wired with inputs and dependencies where needed

# Enable APIs Module - foundational services
module "enable_apis" {
  source     = "./modules/enable_apis"
  project_id = var.project_id
}

# IAM Module - foundational identity and access management
module "iam" {
  source     = "./modules/iam"
  project_id = var.project_id
}

# Networking Module - foundational networking
module "networking" {
  source     = "./modules/networking"
  project_id = var.project_id
  region     = var.region
}

# Storage Module - object storage
module "storage" {
  source     = "./modules/storage"
  project_id = var.project_id
  region     = var.region
}

# Artifact Registry Module - container and package registry
module "artifact_registry" {
  source                    = "./modules/artifact_registry"
  project_id                = var.project_id
  region                    = var.region
  api_enablement_dependency = module.enable_apis
}

# Pub/Sub Module - messaging service
module "pubsub" {
  source     = "./modules/pubsub"
  project_id = var.project_id
}

# BigQuery Module - data warehouse
module "bigquery" {
  source     = "./modules/bigquery"
  project_id = var.project_id
  region     = var.region
}

# Cloud SQL Module - managed relational database
module "cloud_sql" {
  source     = "./modules/cloud_sql"
  project_id = var.project_id
  region     = var.region
}

# GKE Module - managed Kubernetes
module "gke" {
  source     = "./modules/gke"
  project_id = var.project_id
  region     = var.region
}

# Cloud Functions Module - serverless functions
module "cloud_functions" {
  source     = "./modules/cloud_functions"
  project_id = var.project_id
  region     = var.region
}

# Cloud Run Module - containerized applications
module "cloud_run" {
  source     = "./modules/cloud_run"
  project_id = var.project_id
  region     = var.region
}

# Monitoring Module - observability and alerting
module "monitoring" {
  source      = "./modules/monitoring"
  project_id  = var.project_id
  alert_email = var.alert_email
}

# Logging Module - log management
module "logging" {
  source      = "./modules/logging"
  project_id  = var.project_id
  bucket_name = module.storage.bucket_name
}

# Secret Manager Module - secrets management
module "secret_manager" {
  source     = "./modules/secret_manager"
  project_id = var.project_id
}

# KMS Module - key management service
module "kms" {
  source     = "./modules/kms"
  project_id = var.project_id
  location   = var.region
}

# Composer Module - managed Apache Airflow
module "composer" {
  source     = "./modules/composer"
  project_id = var.project_id
  region     = var.region
}

# Dataproc Module - managed Spark and Hadoop
module "dataproc" {
  source     = "./modules/dataproc"
  project_id = var.project_id
  region     = var.region
}

# Workflows Module - serverless orchestration
module "workflows" {
  source     = "./modules/workflows"
  project_id = var.project_id
  region     = var.region
}

# Spanner Module - globally distributed database
module "spanner" {
  source     = "./modules/spanner"
  project_id = var.project_id
  region     = var.region
} 