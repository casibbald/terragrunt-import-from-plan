variable "resource_group_name" {
  description = "The name of the resource group"
  type        = string
}

variable "location" {
  description = "The Azure region where resources will be created"
  type        = string
}

variable "sql_server_name" {
  description = "The name of the SQL server"
  type        = string
}

variable "database_name" {
  description = "The name of the SQL database"
  type        = string
}

variable "sql_server_version" {
  description = "The version of the SQL server"
  type        = string
  default     = "12.0"
}

variable "administrator_login" {
  description = "The administrator login for the SQL server"
  type        = string
}

variable "administrator_login_password" {
  description = "The administrator password for the SQL server"
  type        = string
  sensitive   = true
}

variable "minimum_tls_version" {
  description = "The minimum TLS version for the SQL server"
  type        = string
  default     = "1.2"
}

variable "azuread_admin_login" {
  description = "Azure AD administrator login"
  type        = string
}

variable "azuread_admin_object_id" {
  description = "Azure AD administrator object ID"
  type        = string
}

variable "database_collation" {
  description = "The collation of the database"
  type        = string
  default     = "SQL_Latin1_General_CP1_CI_AS"
}

variable "license_type" {
  description = "License type for the database"
  type        = string
  default     = null
}

variable "max_size_gb" {
  description = "Maximum size of the database in GB"
  type        = number
  default     = 2
}

variable "database_sku_name" {
  description = "SKU name for the database"
  type        = string
  default     = "S0"
}

variable "zone_redundant" {
  description = "Enable zone redundancy"
  type        = bool
  default     = false
}

variable "backup_retention_days" {
  description = "Backup retention in days"
  type        = number
  default     = 7
}

variable "weekly_retention" {
  description = "Weekly backup retention"
  type        = string
  default     = "P1W"
}

variable "monthly_retention" {
  description = "Monthly backup retention"
  type        = string
  default     = "P1M"
}

variable "yearly_retention" {
  description = "Yearly backup retention"
  type        = string
  default     = "P1Y"
}

variable "week_of_year" {
  description = "Week of year for yearly retention"
  type        = number
  default     = 1
}

variable "allowed_ip_ranges" {
  description = "List of allowed IP ranges"
  type        = list(string)
  default     = []
}

variable "enable_auditing" {
  description = "Enable auditing"
  type        = bool
  default     = false
}

variable "audit_storage_endpoint" {
  description = "Storage endpoint for audit logs"
  type        = string
  default     = null
}

variable "audit_storage_account_access_key" {
  description = "Storage account access key for audit logs"
  type        = string
  default     = null
  sensitive   = true
}

variable "audit_retention_in_days" {
  description = "Audit retention in days"
  type        = number
  default     = 30
}

variable "enable_vulnerability_assessment" {
  description = "Enable vulnerability assessment"
  type        = bool
  default     = false
}

variable "va_storage_container_path" {
  description = "Storage container path for vulnerability assessment"
  type        = string
  default     = null
}

variable "va_storage_account_access_key" {
  description = "Storage account access key for vulnerability assessment"
  type        = string
  default     = null
  sensitive   = true
}

variable "va_notification_emails" {
  description = "Email addresses for vulnerability assessment notifications"
  type        = list(string)
  default     = []
}

variable "security_alert_storage_endpoint" {
  description = "Storage endpoint for security alerts"
  type        = string
  default     = null
}

variable "security_alert_storage_account_access_key" {
  description = "Storage account access key for security alerts"
  type        = string
  default     = null
  sensitive   = true
}

variable "security_alert_retention_days" {
  description = "Security alert retention in days"
  type        = number
  default     = 30
}

variable "security_alert_email_addresses" {
  description = "Email addresses for security alerts"
  type        = list(string)
  default     = []
}

variable "enable_private_endpoint" {
  description = "Enable private endpoint"
  type        = bool
  default     = false
}

variable "private_endpoint_subnet_id" {
  description = "Subnet ID for private endpoint"
  type        = string
  default     = null
}

variable "tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
} 