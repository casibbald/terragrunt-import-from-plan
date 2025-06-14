variable "region" {
  type        = string
  description = "AWS region where the bucket should be created"
}

variable "account_id" {
  type        = string
  description = "AWS account ID"
  default     = ""
} 