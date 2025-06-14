output "storage_account_id" {
  description = "The ID of the storage account"
  value       = azurerm_storage_account.example.id
}

output "storage_account_name" {
  description = "The name of the storage account"
  value       = azurerm_storage_account.example.name
}

output "storage_account_primary_endpoint" {
  description = "The primary blob endpoint of the storage account"
  value       = azurerm_storage_account.example.primary_blob_endpoint
}

output "storage_account_primary_access_key" {
  description = "The primary access key for the storage account"
  value       = azurerm_storage_account.example.primary_access_key
  sensitive   = true
}

output "storage_account_connection_string" {
  description = "The connection string for the storage account"
  value       = azurerm_storage_account.example.primary_connection_string
  sensitive   = true
}

output "container_id" {
  description = "The ID of the main storage container"
  value       = azurerm_storage_container.example.id
}

output "container_name" {
  description = "The name of the main storage container"
  value       = azurerm_storage_container.example.name
}

output "logs_container_id" {
  description = "The ID of the logs storage container"
  value       = azurerm_storage_container.logs.id
}

output "backups_container_id" {
  description = "The ID of the backups storage container"
  value       = azurerm_storage_container.backups.id
}

output "static_website_url" {
  description = "The URL of the static website (if enabled)"
  value       = var.enable_static_website ? azurerm_storage_account.example.primary_web_endpoint : null
}

output "blob_url" {
  description = "The URL of the example blob"
  value       = azurerm_storage_blob.example.url
} 