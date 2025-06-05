// variables.tf

variable "project_id" {
  type        = string
  description = "GCP project ID"
}

variable "location" {
  type        = string
  description = "Location for the KMS key ring"
  default     = "global"
}

