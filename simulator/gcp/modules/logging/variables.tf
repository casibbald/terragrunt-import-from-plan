// variables.tf

variable "project_id" {
  type        = string
  description = "GCP project ID"
}

variable "bucket_name" {
  type        = string
  description = "Name of GCS bucket to send log sink data to"
  default     = "simulated-logs-bucket"
}


