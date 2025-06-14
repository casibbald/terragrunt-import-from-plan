# This file includes all module declarations, wired with inputs and dependencies where needed

module "enable_apis" {
  source     = "./modules/enable_apis"
  project_id = var.project_id
}

module "iam" {
  source     = "./modules/iam"
  project_id = var.project_id
}

module "networking" {
  source     = "./modules/networking"
  project_id = var.project_id
  region     = var.region
}

module "storage" {
  source     = "./modules/storage"
  project_id = var.project_id
  region     = var.region
}

module "artifact_registry" {
  source                   = "./modules/artifact_registry"
  project_id               = var.project_id
  region                   = var.region
  api_enablement_dependency = module.enable_apis
}

module "pubsub" {
  source     = "./modules/pubsub"
  project_id = var.project_id
}

module "bigquery" {
  source     = "./modules/bigquery"
  project_id = var.project_id
  region     = var.region
}

module "cloud_sql" {
  source     = "./modules/cloud_sql"
  project_id = var.project_id
  region     = var.region
}

module "gke" {
  source     = "./modules/gke"
  project_id = var.project_id
  region     = var.region
}

module "cloud_functions" {
  source     = "./modules/cloud_functions"
  project_id = var.project_id
  region     = var.region
}

module "cloud_run" {
  source     = "./modules/cloud_run"
  project_id = var.project_id
  region     = var.region
}

module "monitoring" {
  source      = "./modules/monitoring"
  project_id  = var.project_id
  alert_email = "simulate@example.com"
}

module "logging" {
  source       = "./modules/logging"
  project_id   = var.project_id
  bucket_name  = module.storage.bucket_name
}

module "secret_manager" {
  source       = "./modules/secret_manager"
  project_id   = var.project_id
}

module "kms" {
  source     = "./modules/kms"
  project_id = var.project_id
  location   = var.region
}

module "composer" {
  source     = "./modules/composer"
  project_id = var.project_id
  region     = var.region
}

module "dataproc" {
  source     = "./modules/dataproc"
  project_id = var.project_id
  region     = var.region
}

module "workflows" {
  source     = "./modules/workflows"
  project_id = var.project_id
  region     = var.region
}

module "spanner" {
  source     = "./modules/spanner"
  project_id = var.project_id
  region     = var.region
}
