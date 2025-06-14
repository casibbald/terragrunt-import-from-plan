variable "location" {
  description = "The Azure region where resources will be created"
  type        = string
}

variable "resource_group_name" {
  description = "The name of the resource group"
  type        = string
}

variable "application_name" {
  description = "The name of the Azure AD application"
  type        = string
  default     = "example-application"
}

variable "custom_role_name" {
  description = "The name of the custom role"
  type        = string
  default     = "Example Custom Role"
}

variable "managed_identity_name" {
  description = "The name of the managed identity"
  type        = string
  default     = "example-managed-identity"
}

variable "tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
} 