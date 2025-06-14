output "cosmosdb_account_id" {
  description = "The ID of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.id
}

output "cosmosdb_account_name" {
  description = "The name of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.name
}

output "cosmosdb_endpoint" {
  description = "The endpoint of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.endpoint
}

output "cosmosdb_read_endpoints" {
  description = "The read endpoints of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.read_endpoints
}

output "cosmosdb_write_endpoints" {
  description = "The write endpoints of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.write_endpoints
}

output "cosmosdb_primary_key" {
  description = "The primary key of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.primary_key
  sensitive   = true
}

output "cosmosdb_secondary_key" {
  description = "The secondary key of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.secondary_key
  sensitive   = true
}

output "cosmosdb_primary_readonly_key" {
  description = "The primary readonly key of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.primary_readonly_key
  sensitive   = true
}

output "cosmosdb_secondary_readonly_key" {
  description = "The secondary readonly key of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.secondary_readonly_key
  sensitive   = true
}

output "cosmosdb_connection_strings" {
  description = "The connection strings of the Cosmos DB account"
  value       = azurerm_cosmosdb_account.example.connection_strings
  sensitive   = true
}

output "sql_database_id" {
  description = "The ID of the SQL database (if created)"
  value       = var.kind == "GlobalDocumentDB" && var.create_sql_database ? azurerm_cosmosdb_sql_database.example[0].id : null
}

output "sql_container_id" {
  description = "The ID of the SQL container (if created)"
  value       = var.kind == "GlobalDocumentDB" && var.create_sql_container ? azurerm_cosmosdb_sql_container.example[0].id : null
}

output "mongo_database_id" {
  description = "The ID of the MongoDB database (if created)"
  value       = var.kind == "MongoDB" && var.create_mongo_database ? azurerm_cosmosdb_mongo_database.example[0].id : null
}

output "mongo_collection_id" {
  description = "The ID of the MongoDB collection (if created)"
  value       = var.kind == "MongoDB" && var.create_mongo_collection ? azurerm_cosmosdb_mongo_collection.example[0].id : null
}

output "private_endpoint_id" {
  description = "The ID of the private endpoint (if enabled)"
  value       = var.enable_private_endpoint ? azurerm_private_endpoint.cosmos_private_endpoint[0].id : null
}

output "identity_principal_id" {
  description = "The principal ID of the Cosmos DB account identity"
  value       = azurerm_cosmosdb_account.example.identity[0].principal_id
} 