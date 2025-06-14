# Resource Group
resource "azurerm_resource_group" "main" {
  name     = var.resource_group_name
  location = var.location

  tags = var.tags
}

# Log Analytics Workspace (required for AKS)
resource "azurerm_log_analytics_workspace" "main" {
  name                = "${var.project_name}-log-analytics"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  sku                 = "PerGB2018"
  retention_in_days   = 30

  tags = var.tags
}

# Networking Module
module "networking" {
  source = "./modules/networking"

  resource_group_name   = azurerm_resource_group.main.name
  location              = azurerm_resource_group.main.location
  vnet_name             = "${var.project_name}-vnet"
  vnet_address_space    = var.vnet_address_space
  public_subnet_prefix  = [var.subnet_address_prefixes.public]
  private_subnet_prefix = [var.subnet_address_prefixes.private]
  tags                  = var.tags
}

# IAM Module
module "iam" {
  source = "./modules/iam"

  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  tags                = var.tags
}

# Storage Module
module "storage" {
  source = "./modules/storage"

  resource_group_name  = azurerm_resource_group.main.name
  location             = azurerm_resource_group.main.location
  storage_account_name = var.storage_account_name
  container_name       = var.container_name
  tags                 = var.tags
}

# Key Vault Module
module "key_vault" {
  source = "./modules/key_vault"

  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  key_vault_name      = var.key_vault_name
  tags                = var.tags
}

# Functions Module
module "functions" {
  source = "./modules/functions"

  resource_group_name           = azurerm_resource_group.main.name
  location                      = azurerm_resource_group.main.location
  function_app_name             = var.function_app_name
  function_storage_account_name = var.function_storage_account_name
  service_plan_name             = "${var.function_app_name}-plan"
  tags                          = var.tags
}

# SQL Module
module "sql" {
  source = "./modules/sql"

  resource_group_name          = azurerm_resource_group.main.name
  location                     = azurerm_resource_group.main.location
  sql_server_name              = var.sql_server_name
  database_name                = var.database_name
  administrator_login          = var.administrator_login
  administrator_login_password = var.administrator_login_password
  azuread_admin_login          = var.azuread_admin_login
  azuread_admin_object_id      = var.azuread_admin_object_id
  tags                         = var.tags
}

# Container Registry Module
module "container_registry" {
  source = "./modules/container_registry"

  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  registry_name       = var.container_registry_name
  tags                = var.tags
}

# AKS Module
module "aks" {
  source = "./modules/aks"

  resource_group_name        = azurerm_resource_group.main.name
  location                   = azurerm_resource_group.main.location
  cluster_name               = var.aks_cluster_name
  dns_prefix                 = var.aks_dns_prefix
  node_count                 = var.aks_node_count
  subnet_id                  = module.networking.private_subnet_id
  log_analytics_workspace_id = azurerm_log_analytics_workspace.main.id
  container_registry_id      = module.container_registry.registry_id
  tags                       = var.tags

  depends_on = [module.networking]
}

# Cosmos DB Module
module "cosmos_db" {
  source = "./modules/cosmos_db"

  resource_group_name   = azurerm_resource_group.main.name
  location              = azurerm_resource_group.main.location
  cosmosdb_account_name = var.cosmosdb_account_name
  sql_database_name     = var.cosmosdb_database_name
  geo_locations = [
    {
      location          = azurerm_resource_group.main.location
      failover_priority = 0
      zone_redundant    = false
    }
  ]
  tags = var.tags
} 