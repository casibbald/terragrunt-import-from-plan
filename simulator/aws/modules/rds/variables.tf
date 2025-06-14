variable "region" {
  type        = string
  description = "AWS region where the RDS instance should be created"
}

variable "subnet_ids" {
  type        = list(string)
  description = "List of subnet IDs for the DB subnet group"
}

variable "security_group_ids" {
  type        = list(string)
  description = "List of security group IDs for the RDS instance"
} 