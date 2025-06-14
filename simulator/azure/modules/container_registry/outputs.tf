output "registry_id" {
  description = "The ID of the container registry"
  value       = azurerm_container_registry.example.id
}

output "registry_name" {
  description = "The name of the container registry"
  value       = azurerm_container_registry.example.name
}

output "login_server" {
  description = "The URL that can be used to log into the container registry"
  value       = azurerm_container_registry.example.login_server
}

output "admin_username" {
  description = "The admin username for the container registry"
  value       = var.admin_enabled ? azurerm_container_registry.example.admin_username : null
}

output "admin_password" {
  description = "The admin password for the container registry"
  value       = var.admin_enabled ? azurerm_container_registry.example.admin_password : null
  sensitive   = true
}

output "identity_principal_id" {
  description = "The principal ID of the container registry identity"
  value       = azurerm_container_registry.example.identity[0].principal_id
}

output "identity_tenant_id" {
  description = "The tenant ID of the container registry identity"
  value       = azurerm_container_registry.example.identity[0].tenant_id
}

output "private_endpoint_id" {
  description = "The ID of the private endpoint (if enabled)"
  value       = var.enable_private_endpoint && var.sku == "Premium" ? azurerm_private_endpoint.acr_private_endpoint[0].id : null
}

output "webhook_id" {
  description = "The ID of the webhook (if enabled)"
  value       = var.enable_webhook ? azurerm_container_registry_webhook.example[0].id : null
}

output "scope_map_id" {
  description = "The ID of the scope map (if enabled)"
  value       = var.sku == "Premium" && var.enable_scope_map ? azurerm_container_registry_scope_map.example[0].id : null
}

output "token_id" {
  description = "The ID of the token (if enabled)"
  value       = var.sku == "Premium" && var.enable_token ? azurerm_container_registry_token.example[0].id : null
} 