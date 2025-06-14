variable "resource_group_name" {
  description = "The name of the resource group"
  type        = string
}

variable "location" {
  description = "The Azure region where resources will be created"
  type        = string
}

variable "registry_name" {
  description = "The name of the container registry"
  type        = string
}

variable "sku" {
  description = "The SKU of the container registry"
  type        = string
  default     = "Standard"
  validation {
    condition     = contains(["Basic", "Standard", "Premium"], var.sku)
    error_message = "SKU must be Basic, Standard, or Premium."
  }
}

variable "admin_enabled" {
  description = "Enable admin user for the container registry"
  type        = bool
  default     = false
}

variable "public_network_access_enabled" {
  description = "Enable public network access"
  type        = bool
  default     = true
}

variable "network_rule_set_enabled" {
  description = "Enable network rule set"
  type        = bool
  default     = false
}

variable "network_rule_default_action" {
  description = "Default action for network rules"
  type        = string
  default     = "Allow"
  validation {
    condition     = contains(["Allow", "Deny"], var.network_rule_default_action)
    error_message = "Default action must be Allow or Deny."
  }
}

variable "allowed_ip_ranges" {
  description = "List of allowed IP ranges"
  type        = list(string)
  default     = []
}

variable "allowed_subnet_ids" {
  description = "List of allowed subnet IDs"
  type        = list(string)
  default     = []
}

variable "georeplications" {
  description = "List of georeplications for Premium SKU"
  type = list(object({
    location                  = string
    zone_redundancy_enabled   = bool
    regional_endpoint_enabled = bool
    tags                      = map(string)
  }))
  default = []
}

variable "encryption_enabled" {
  description = "Enable encryption for Premium SKU"
  type        = bool
  default     = false
}

variable "encryption_key_vault_key_id" {
  description = "Key Vault key ID for encryption"
  type        = string
  default     = null
}

variable "encryption_identity_client_id" {
  description = "Client ID for encryption identity"
  type        = string
  default     = null
}

variable "retention_policy_enabled" {
  description = "Enable retention policy for Premium SKU"
  type        = bool
  default     = false
}

variable "retention_policy_days" {
  description = "Number of days for retention policy"
  type        = number
  default     = 7
}

variable "trust_policy_enabled" {
  description = "Enable trust policy for Premium SKU"
  type        = bool
  default     = false
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

variable "private_dns_zone_ids" {
  description = "List of private DNS zone IDs"
  type        = list(string)
  default     = []
}

variable "pull_role_assignment_principal_ids" {
  description = "List of principal IDs for AcrPull role"
  type        = list(string)
  default     = []
}

variable "push_role_assignment_principal_ids" {
  description = "List of principal IDs for AcrPush role"
  type        = list(string)
  default     = []
}

variable "enable_webhook" {
  description = "Enable webhook"
  type        = bool
  default     = false
}

variable "webhook_name" {
  description = "Name of the webhook"
  type        = string
  default     = "example-webhook"
}

variable "webhook_service_uri" {
  description = "Service URI for webhook"
  type        = string
  default     = "https://example.com/webhook"
}

variable "webhook_scope" {
  description = "Scope for webhook"
  type        = string
  default     = "*"
}

variable "webhook_actions" {
  description = "Actions for webhook"
  type        = list(string)
  default     = ["push"]
}

variable "webhook_custom_headers" {
  description = "Custom headers for webhook"
  type        = map(string)
  default     = {}
}

variable "enable_scope_map" {
  description = "Enable scope map for Premium SKU"
  type        = bool
  default     = false
}

variable "scope_map_name" {
  description = "Name of the scope map"
  type        = string
  default     = "example-scope-map"
}

variable "scope_map_actions" {
  description = "Actions for scope map"
  type        = list(string)
  default     = ["repositories/*/content/read", "repositories/*/content/write"]
}

variable "enable_token" {
  description = "Enable token for Premium SKU"
  type        = bool
  default     = false
}

variable "token_name" {
  description = "Name of the token"
  type        = string
  default     = "example-token"
}

variable "tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
} 