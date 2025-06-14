variable "project_id" {
  description = "The Google Cloud project ID"
  type        = string
}

variable "region" {
  description = "The Google Cloud region where resources will be created"
  type        = string
  default     = "europe-west1"
}

variable "zone" {
  description = "The Google Cloud zone where resources will be created"
  type        = string
  default     = ""
}

variable "alert_email" {
  description = "Email address for monitoring alerts"
  type        = string
  default     = "admin@example.com"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "development"
}

variable "labels" {
  description = "Additional labels to apply to resources"
  type        = map(string)
  default = {
    project    = "terraform-simulator"
    managed_by = "terraform"
  }
} 