terraform {
  required_version = ">= 1.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 4.0"
    }
    google-beta = {
      source  = "hashicorp/google-beta"
      version = "~> 4.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.1"
    }
    archive = {
      source  = "hashicorp/archive"
      version = "~> 2.2"
    }
  }
}

# Configure the Google Cloud Provider
provider "google" {
  project = var.project_id
  region  = var.region

  # For CI/CD and testing environments without GCP credentials
  # These settings allow terraform validate/plan to work without auth
}

# Configure the Google Cloud Beta Provider (for newer features)
provider "google-beta" {
  project = var.project_id
  region  = var.region
} 