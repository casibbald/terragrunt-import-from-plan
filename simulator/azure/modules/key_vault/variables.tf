variable "resource_group_name" {
  description = "The name of the resource group"
  type        = string
}

variable "location" {
  description = "The Azure region where resources will be created"
  type        = string
}

variable "key_vault_name" {
  description = "The name of the Key Vault (must be globally unique)"
  type        = string
}

variable "sku_name" {
  description = "The SKU name for the Key Vault"
  type        = string
  default     = "standard"
  validation {
    condition     = contains(["standard", "premium"], var.sku_name)
    error_message = "SKU name must be either 'standard' or 'premium'."
  }
}

variable "enabled_for_disk_encryption" {
  description = "Enable Azure Disk Encryption to retrieve secrets from the vault"
  type        = bool
  default     = true
}

variable "enabled_for_deployment" {
  description = "Enable Azure Virtual Machines to retrieve certificates stored as secrets from the vault"
  type        = bool
  default     = true
}

variable "enabled_for_template_deployment" {
  description = "Enable Azure Resource Manager to retrieve secrets from the vault"
  type        = bool
  default     = true
}

variable "purge_protection_enabled" {
  description = "Enable purge protection for the Key Vault"
  type        = bool
  default     = false
}

variable "soft_delete_retention_days" {
  description = "Number of days to retain deleted items"
  type        = number
  default     = 7
  validation {
    condition     = var.soft_delete_retention_days >= 7 && var.soft_delete_retention_days <= 90
    error_message = "Soft delete retention days must be between 7 and 90."
  }
}

variable "default_action" {
  description = "Default action for network access"
  type        = string
  default     = "Allow"
  validation {
    condition     = contains(["Allow", "Deny"], var.default_action)
    error_message = "Default action must be either 'Allow' or 'Deny'."
  }
}

variable "allowed_ip_ranges" {
  description = "List of IP ranges allowed to access the Key Vault"
  type        = list(string)
  default     = []
}

variable "secret_value" {
  description = "The value of the example secret"
  type        = string
  default     = "example-secret-value"
  sensitive   = true
}

variable "enable_logging" {
  description = "Enable diagnostic logging for Key Vault"
  type        = bool
  default     = false
}

variable "log_storage_account_id" {
  description = "Storage account ID for diagnostic logs"
  type        = string
  default     = null
}

variable "tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
} 