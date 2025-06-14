terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
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

# Configure the AWS Provider
provider "aws" {
  region = var.region

  # For CI/CD and testing environments without AWS credentials
  skip_credentials_validation = true
  skip_metadata_api_check     = true
  skip_region_validation      = true
  skip_requesting_account_id  = true

  # Use fake credentials for CI/CD to prevent credential lookups
  access_key = "mock_access_key"
  secret_key = "mock_secret_key"

  # Default tags for all resources
  default_tags {
    tags = {
      Project     = "terraform-simulator"
      ManagedBy   = "terraform"
      Environment = "development"
    }
  }
} 