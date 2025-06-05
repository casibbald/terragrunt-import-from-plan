// variables.tf

variable "project_id" {
  type        = string
  description = "GCP project ID"
}

variable "region" {
  type        = string
  description = "Region for the Artifact Registry"
}

variable "api_enablement_dependency" {
  description = "Explicit dependency on API enablement module"
  type        = any
}
