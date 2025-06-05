
// variables.tf

variable "project_id" {
  type        = string
  description = "GCP project ID"
}

variable "alert_email" {
  type        = string
  description = "Email address for monitoring alerts"
  default     = "simulate@example.com"
}
