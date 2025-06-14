variable "resource_group_name" {
  description = "The name of the resource group"
  type        = string
}

variable "location" {
  description = "The Azure region where resources will be created"
  type        = string
}

variable "function_app_name" {
  description = "The name of the Function App"
  type        = string
}

variable "function_storage_account_name" {
  description = "The name of the storage account for Function App"
  type        = string
}

variable "service_plan_name" {
  description = "The name of the App Service Plan"
  type        = string
}

variable "os_type" {
  description = "The operating system type for the Function App"
  type        = string
  default     = "Linux"
  validation {
    condition     = contains(["Linux", "Windows"], var.os_type)
    error_message = "OS type must be either 'Linux' or 'Windows'."
  }
}

variable "sku_name" {
  description = "The SKU name for the App Service Plan"
  type        = string
  default     = "Y1"
}

variable "runtime" {
  description = "The runtime for the Function App"
  type        = string
  default     = "node"
  validation {
    condition     = contains(["node", "python", "dotnet", "java"], var.runtime)
    error_message = "Runtime must be one of: node, python, dotnet, java."
  }
}

variable "runtime_version" {
  description = "The version of the runtime"
  type        = string
  default     = "18"
}

variable "always_on" {
  description = "Should the Function App be always on"
  type        = bool
  default     = false
}

variable "app_settings" {
  description = "Additional application settings for the Function App"
  type        = map(string)
  default     = {}
}

variable "cors_allowed_origins" {
  description = "List of allowed origins for CORS"
  type        = list(string)
  default     = ["*"]
}

variable "cors_support_credentials" {
  description = "Should CORS support credentials"
  type        = bool
  default     = false
}

variable "enable_application_insights" {
  description = "Enable Application Insights for monitoring"
  type        = bool
  default     = true
}

variable "create_example_function" {
  description = "Create an example HTTP triggered function"
  type        = bool
  default     = true
}

variable "tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
} 