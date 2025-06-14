// locals.tf

locals {
  required_services = [
    "compute.googleapis.com",
    "artifactregistry.googleapis.com",
    "pubsub.googleapis.com",
    "bigquery.googleapis.com",
    "sqladmin.googleapis.com",
    "container.googleapis.com",
    "cloudfunctions.googleapis.com",
    "run.googleapis.com",
    "monitoring.googleapis.com",
    "logging.googleapis.com",
    "secretmanager.googleapis.com",
    "cloudkms.googleapis.com",
    "composer.googleapis.com",
    "dataproc.googleapis.com",
    "workflows.googleapis.com",
    "spanner.googleapis.com",
    "iam.googleapis.com",
    "servicemanagement.googleapis.com"
  ]
}