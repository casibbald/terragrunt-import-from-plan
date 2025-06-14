output "sql_server_id" {
  description = "The ID of the SQL server"
  value       = azurerm_mssql_server.example.id
}

output "sql_server_name" {
  description = "The name of the SQL server"
  value       = azurerm_mssql_server.example.name
}

output "sql_server_fqdn" {
  description = "The fully qualified domain name of the SQL server"
  value       = azurerm_mssql_server.example.fully_qualified_domain_name
}

output "sql_database_id" {
  description = "The ID of the SQL database"
  value       = azurerm_mssql_database.example.id
}

output "sql_database_name" {
  description = "The name of the SQL database"
  value       = azurerm_mssql_database.example.name
}

output "connection_string" {
  description = "The connection string for the SQL database"
  value       = "Server=tcp:${azurerm_mssql_server.example.fully_qualified_domain_name},1433;Initial Catalog=${azurerm_mssql_database.example.name};Persist Security Info=False;User ID=${azurerm_mssql_server.example.administrator_login};MultipleActiveResultSets=False;Encrypt=True;TrustServerCertificate=False;Connection Timeout=30;"
  sensitive   = true
}

output "private_endpoint_id" {
  description = "The ID of the private endpoint (if enabled)"
  value       = var.enable_private_endpoint ? azurerm_private_endpoint.sql_private_endpoint[0].id : null
}

output "firewall_rule_ids" {
  description = "The IDs of the firewall rules"
  value       = concat([azurerm_mssql_firewall_rule.azure_services.id], azurerm_mssql_firewall_rule.allowed_ips[*].id)
} 