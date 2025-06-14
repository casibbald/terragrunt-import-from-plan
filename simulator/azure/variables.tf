variable "location" {
  description = "The Azure region where resources will be created"
  type        = string
  default     = "East US"
}

variable "resource_group_name" {
  description = "The name of the resource group"
  type        = string
  default     = "example-rg"
}

variable "environment" {
  description = "The environment name"
  type        = string
  default     = "development"
}

variable "project_name" {
  description = "The name of the project"
  type        = string
  default     = "azure-simulator"
}

# Networking Variables
variable "vnet_address_space" {
  description = "Address space for the virtual network"
  type        = list(string)
  default     = ["10.0.0.0/16"]
}

variable "subnet_address_prefixes" {
  description = "Address prefixes for subnets"
  type        = map(string)
  default = {
    public  = "10.0.1.0/24"
    private = "10.0.2.0/24"
    aks     = "10.0.3.0/24"
  }
}

# Storage Variables
variable "storage_account_name" {
  description = "The name of the storage account (must be globally unique)"
  type        = string
  default     = "examplestorageacct123"
}

variable "container_name" {
  description = "The name of the storage container"
  type        = string
  default     = "example-container"
}

# Key Vault Variables
variable "key_vault_name" {
  description = "The name of the Key Vault (must be globally unique)"
  type        = string
  default     = "example-keyvault-123"
}

# Function App Variables
variable "function_app_name" {
  description = "The name of the Function App"
  type        = string
  default     = "example-function-app"
}

variable "function_storage_account_name" {
  description = "The name of the storage account for Function App"
  type        = string
  default     = "examplefuncstore123"
}

# SQL Variables
variable "sql_server_name" {
  description = "The name of the SQL server"
  type        = string
  default     = "example-sql-server"
}

variable "database_name" {
  description = "The name of the SQL database"
  type        = string
  default     = "example-database"
}

variable "administrator_login" {
  description = "The administrator login for the SQL server"
  type        = string
  default     = "sqladmin"
}

variable "administrator_login_password" {
  description = "The administrator password for the SQL server"
  type        = string
  default     = "P@ssw0rd123!"
  sensitive   = true
}

# AKS Variables
variable "aks_cluster_name" {
  description = "The name of the AKS cluster"
  type        = string
  default     = "example-aks-cluster"
}

variable "aks_dns_prefix" {
  description = "DNS prefix for the AKS cluster"
  type        = string
  default     = "example-aks"
}

variable "aks_node_count" {
  description = "Number of nodes in the AKS cluster"
  type        = number
  default     = 2
}

# Container Registry Variables
variable "container_registry_name" {
  description = "The name of the container registry"
  type        = string
  default     = "exampleacr123"
}

# Cosmos DB Variables
variable "cosmosdb_account_name" {
  description = "The name of the Cosmos DB account"
  type        = string
  default     = "example-cosmosdb-account"
}

variable "cosmosdb_database_name" {
  description = "The name of the Cosmos DB database"
  type        = string
  default     = "example-database"
}

# Azure AD Variables
variable "azuread_admin_login" {
  description = "Azure AD admin login"
  type        = string
  default     = "azureadmin@example.com"
}

variable "azuread_admin_object_id" {
  description = "Azure AD admin object ID"
  type        = string
  default     = "00000000-0000-0000-0000-000000000000"
}

# Common Tags
variable "tags" {
  description = "Common tags to apply to all resources"
  type        = map(string)
  default = {
    Environment = "development"
    Project     = "azure-simulator"
    Owner       = "terraform"
  }
} 