output "key_vault_id" {
  description = "The ID of the Key Vault"
  value       = azurerm_key_vault.example.id
}

output "key_vault_name" {
  description = "The name of the Key Vault"
  value       = azurerm_key_vault.example.name
}

output "key_vault_uri" {
  description = "The URI of the Key Vault"
  value       = azurerm_key_vault.example.vault_uri
}

output "key_id" {
  description = "The ID of the example key"
  value       = azurerm_key_vault_key.example.id
}

output "key_version" {
  description = "The current version of the example key"
  value       = azurerm_key_vault_key.example.version
}

output "secret_id" {
  description = "The ID of the example secret"
  value       = azurerm_key_vault_secret.example.id
}

output "secret_version" {
  description = "The current version of the example secret"
  value       = azurerm_key_vault_secret.example.version
}

output "certificate_id" {
  description = "The ID of the example certificate"
  value       = azurerm_key_vault_certificate.example.id
}

output "certificate_thumbprint" {
  description = "The thumbprint of the certificate"
  value       = azurerm_key_vault_certificate.example.thumbprint
}

output "certificate_secret_id" {
  description = "The secret ID of the certificate"
  value       = azurerm_key_vault_certificate.example.secret_id
} 