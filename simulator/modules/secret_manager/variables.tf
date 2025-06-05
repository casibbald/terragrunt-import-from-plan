// variables.tf

variable "project_id" {
  type        = string
  description = "GCP project ID"
}

variable "secret_value" {
  type        = string
  description = "The simulated secret value"
  default     = "s3cr3t"
}
