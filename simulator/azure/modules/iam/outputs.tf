output "application_id" {
  description = "The Application ID of the Azure AD application"
  value       = azuread_application.example.application_id
}

output "service_principal_id" {
  description = "The Object ID of the service principal"
  value       = azuread_service_principal.example.object_id
}

output "service_principal_application_id" {
  description = "The Application ID of the service principal"
  value       = azuread_service_principal.example.application_id
}

output "client_secret" {
  description = "The client secret for the service principal"
  value       = azuread_service_principal_password.example.value
  sensitive   = true
}

output "custom_role_id" {
  description = "The ID of the custom role definition"
  value       = azurerm_role_definition.example.role_definition_resource_id
}

output "managed_identity_id" {
  description = "The ID of the user-assigned managed identity"
  value       = azurerm_user_assigned_identity.example.id
}

output "managed_identity_principal_id" {
  description = "The principal ID of the managed identity"
  value       = azurerm_user_assigned_identity.example.principal_id
}

output "managed_identity_client_id" {
  description = "The client ID of the managed identity"
  value       = azurerm_user_assigned_identity.example.client_id
} 