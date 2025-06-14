# Resource Group Outputs
output "resource_group_id" {
  description = "The ID of the resource group"
  value       = azurerm_resource_group.main.id
}

output "resource_group_name" {
  description = "The name of the resource group"
  value       = azurerm_resource_group.main.name
}

output "location" {
  description = "The Azure region"
  value       = azurerm_resource_group.main.location
}

# Networking Outputs
output "vnet_id" {
  description = "The ID of the virtual network"
  value       = module.networking.vnet_id
}

output "public_subnet_id" {
  description = "The ID of the public subnet"
  value       = module.networking.public_subnet_id
}

output "private_subnet_id" {
  description = "The ID of the private subnet"
  value       = module.networking.private_subnet_id
}

# IAM Outputs
output "service_principal_id" {
  description = "The service principal object ID"
  value       = module.iam.service_principal_id
}

output "managed_identity_id" {
  description = "The managed identity ID"
  value       = module.iam.managed_identity_id
}

# Storage Outputs
output "storage_account_id" {
  description = "The ID of the storage account"
  value       = module.storage.storage_account_id
}

output "storage_account_name" {
  description = "The name of the storage account"
  value       = module.storage.storage_account_name
}

output "storage_account_primary_endpoint" {
  description = "The primary blob endpoint"
  value       = module.storage.storage_account_primary_endpoint
}

# Key Vault Outputs
output "key_vault_id" {
  description = "The ID of the Key Vault"
  value       = module.key_vault.key_vault_id
}

output "key_vault_uri" {
  description = "The URI of the Key Vault"
  value       = module.key_vault.key_vault_uri
}

# Functions Outputs
output "function_app_id" {
  description = "The ID of the Function App"
  value       = module.functions.function_app_id
}

output "function_app_url" {
  description = "The URL of the Function App"
  value       = module.functions.function_app_url
}

# SQL Outputs
output "sql_server_id" {
  description = "The ID of the SQL server"
  value       = module.sql.sql_server_id
}

output "sql_database_id" {
  description = "The ID of the SQL database"
  value       = module.sql.sql_database_id
}

# Container Registry Outputs
output "container_registry_id" {
  description = "The ID of the container registry"
  value       = module.container_registry.registry_id
}

output "container_registry_login_server" {
  description = "The login server URL for the container registry"
  value       = module.container_registry.login_server
}

# AKS Outputs
output "aks_cluster_id" {
  description = "The ID of the AKS cluster"
  value       = module.aks.cluster_id
}

output "aks_cluster_name" {
  description = "The name of the AKS cluster"
  value       = module.aks.cluster_name
}

output "aks_kube_config" {
  description = "The Kubernetes configuration for the AKS cluster"
  value       = module.aks.kube_config
  sensitive   = true
}

# Cosmos DB Outputs
output "cosmosdb_account_id" {
  description = "The ID of the Cosmos DB account"
  value       = module.cosmos_db.cosmosdb_account_id
}

output "cosmosdb_endpoint" {
  description = "The endpoint of the Cosmos DB account"
  value       = module.cosmos_db.cosmosdb_endpoint
}

# Log Analytics Outputs
output "log_analytics_workspace_id" {
  description = "The ID of the Log Analytics workspace"
  value       = azurerm_log_analytics_workspace.main.id
}

output "log_analytics_workspace_name" {
  description = "The name of the Log Analytics workspace"
  value       = azurerm_log_analytics_workspace.main.name
} 