output "function_app_id" {
  description = "The ID of the Function App"
  value       = var.os_type == "Linux" ? (length(azurerm_linux_function_app.example) > 0 ? azurerm_linux_function_app.example[0].id : null) : (length(azurerm_windows_function_app.example) > 0 ? azurerm_windows_function_app.example[0].id : null)
}

output "function_app_name" {
  description = "The name of the Function App"
  value       = var.function_app_name
}

output "function_app_default_hostname" {
  description = "The default hostname of the Function App"
  value       = var.os_type == "Linux" ? (length(azurerm_linux_function_app.example) > 0 ? azurerm_linux_function_app.example[0].default_hostname : null) : (length(azurerm_windows_function_app.example) > 0 ? azurerm_windows_function_app.example[0].default_hostname : null)
}

output "function_app_url" {
  description = "The URL of the Function App"
  value       = var.os_type == "Linux" ? (length(azurerm_linux_function_app.example) > 0 ? "https://${azurerm_linux_function_app.example[0].default_hostname}" : null) : (length(azurerm_windows_function_app.example) > 0 ? "https://${azurerm_windows_function_app.example[0].default_hostname}" : null)
}

output "function_app_identity_principal_id" {
  description = "The principal ID of the Function App's managed identity"
  value       = var.os_type == "Linux" ? (length(azurerm_linux_function_app.example) > 0 ? azurerm_linux_function_app.example[0].identity[0].principal_id : null) : (length(azurerm_windows_function_app.example) > 0 ? azurerm_windows_function_app.example[0].identity[0].principal_id : null)
}

output "storage_account_id" {
  description = "The ID of the storage account used by the Function App"
  value       = azurerm_storage_account.functions.id
}

output "storage_account_name" {
  description = "The name of the storage account used by the Function App"
  value       = azurerm_storage_account.functions.name
}

output "service_plan_id" {
  description = "The ID of the App Service Plan"
  value       = azurerm_service_plan.functions.id
}

output "application_insights_id" {
  description = "The ID of Application Insights (if enabled)"
  value       = var.enable_application_insights ? azurerm_application_insights.functions[0].id : null
}

output "application_insights_instrumentation_key" {
  description = "The instrumentation key of Application Insights (if enabled)"
  value       = var.enable_application_insights ? azurerm_application_insights.functions[0].instrumentation_key : null
  sensitive   = true
} 