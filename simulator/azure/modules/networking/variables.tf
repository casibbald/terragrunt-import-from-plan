variable "location" {
  description = "The Azure region where resources will be created"
  type        = string
  default     = "East US"
}

variable "resource_group_name" {
  description = "The name of the resource group"
  type        = string
  default     = "example-resources"
}

variable "vnet_name" {
  description = "The name of the virtual network"
  type        = string
  default     = "example-vnet"
}

variable "vnet_address_space" {
  description = "The address space for the virtual network"
  type        = list(string)
  default     = ["10.0.0.0/16"]
}

variable "public_subnet_prefix" {
  description = "The address prefix for the public subnet"
  type        = list(string)
  default     = ["10.0.1.0/24"]
}

variable "private_subnet_prefix" {
  description = "The address prefix for the private subnet"
  type        = list(string)
  default     = ["10.0.2.0/24"]
}

variable "environment" {
  description = "The environment tag"
  type        = string
  default     = "development"
}

variable "tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
} 