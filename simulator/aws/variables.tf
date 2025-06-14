variable "region" {
  description = "The AWS region to deploy resources in"
  type        = string
  default     = "us-east-1"
}

variable "account_id" {
  description = "AWS Account ID"
  type        = string
  default     = "123456789012"  # Mock account ID for CI/CD
}

variable "ci_mode" {
  description = "Enable CI/CD mode (disables data sources that require API calls)"
  type        = bool
  default     = true  # Default to CI mode for safety
} 