# Cosmos DB Account
resource "azurerm_cosmosdb_account" "example" {
  name                            = var.cosmosdb_account_name
  location                        = var.location
  resource_group_name             = var.resource_group_name
  offer_type                      = "Standard"
  kind                            = var.kind
  enable_automatic_failover       = var.enable_automatic_failover
  enable_multiple_write_locations = var.enable_multiple_write_locations

  # Consistency policy
  consistency_policy {
    consistency_level       = var.consistency_level
    max_interval_in_seconds = var.consistency_level == "BoundedStaleness" ? var.max_interval_in_seconds : null
    max_staleness_prefix    = var.consistency_level == "BoundedStaleness" ? var.max_staleness_prefix : null
  }

  # Geographic locations for replication
  dynamic "geo_location" {
    for_each = var.geo_locations
    content {
      location          = geo_location.value.location
      failover_priority = geo_location.value.failover_priority
      zone_redundant    = geo_location.value.zone_redundant
    }
  }

  # Capabilities
  dynamic "capabilities" {
    for_each = var.capabilities
    content {
      name = capabilities.value
    }
  }

  # Virtual network rule
  dynamic "virtual_network_rule" {
    for_each = var.subnet_ids
    content {
      id = virtual_network_rule.value
    }
  }

  # IP range filter
  ip_range_filter = join(",", var.allowed_ip_ranges)

  # Access key metadata writes
  is_virtual_network_filter_enabled = var.enable_virtual_network_filter
  public_network_access_enabled     = var.public_network_access_enabled

  # Backup
  backup {
    type                = var.backup_type
    interval_in_minutes = var.backup_type == "Periodic" ? var.backup_interval_in_minutes : null
    retention_in_hours  = var.backup_type == "Periodic" ? var.backup_retention_in_hours : null
    storage_redundancy  = var.backup_type == "Periodic" ? var.backup_storage_redundancy : null
  }

  # CORS
  dynamic "cors_rule" {
    for_each = var.enable_cors ? var.cors_rules : []
    content {
      allowed_headers    = cors_rule.value.allowed_headers
      allowed_methods    = cors_rule.value.allowed_methods
      allowed_origins    = cors_rule.value.allowed_origins
      exposed_headers    = cors_rule.value.exposed_headers
      max_age_in_seconds = cors_rule.value.max_age_in_seconds
    }
  }

  # Identity
  identity {
    type = "SystemAssigned"
  }

  tags = merge(var.tags, {
    Name        = var.cosmosdb_account_name
    Environment = "development"
  })
}

# SQL Database (if kind is GlobalDocumentDB)
resource "azurerm_cosmosdb_sql_database" "example" {
  count               = var.kind == "GlobalDocumentDB" && var.create_sql_database ? 1 : 0
  name                = var.sql_database_name
  resource_group_name = var.resource_group_name
  account_name        = azurerm_cosmosdb_account.example.name
  throughput          = var.sql_database_throughput

  autoscale_settings {
    max_throughput = var.sql_database_autoscale_max_throughput
  }
}

# SQL Container
resource "azurerm_cosmosdb_sql_container" "example" {
  count               = var.kind == "GlobalDocumentDB" && var.create_sql_container ? 1 : 0
  name                = var.sql_container_name
  resource_group_name = var.resource_group_name
  account_name        = azurerm_cosmosdb_account.example.name
  database_name       = var.create_sql_database ? azurerm_cosmosdb_sql_database.example[0].name : var.existing_sql_database_name
  partition_key_path  = var.partition_key_path
  throughput          = var.sql_container_throughput

  # Indexing policy
  indexing_policy {
    indexing_mode = "consistent"

    dynamic "included_path" {
      for_each = var.included_paths
      content {
        path = included_path.value
      }
    }

    dynamic "excluded_path" {
      for_each = var.excluded_paths
      content {
        path = excluded_path.value
      }
    }
  }

  # Unique key policy
  dynamic "unique_key" {
    for_each = var.unique_keys
    content {
      paths = unique_key.value
    }
  }

  # Conflict resolution policy
  conflict_resolution_policy {
    mode                     = var.conflict_resolution_mode
    conflict_resolution_path = var.conflict_resolution_path
  }

  autoscale_settings {
    max_throughput = var.sql_container_autoscale_max_throughput
  }
}

# MongoDB Database (if kind is MongoDB)
resource "azurerm_cosmosdb_mongo_database" "example" {
  count               = var.kind == "MongoDB" && var.create_mongo_database ? 1 : 0
  name                = var.mongo_database_name
  resource_group_name = var.resource_group_name
  account_name        = azurerm_cosmosdb_account.example.name
  throughput          = var.mongo_database_throughput

  autoscale_settings {
    max_throughput = var.mongo_database_autoscale_max_throughput
  }
}

# MongoDB Collection
resource "azurerm_cosmosdb_mongo_collection" "example" {
  count               = var.kind == "MongoDB" && var.create_mongo_collection ? 1 : 0
  name                = var.mongo_collection_name
  resource_group_name = var.resource_group_name
  account_name        = azurerm_cosmosdb_account.example.name
  database_name       = var.create_mongo_database ? azurerm_cosmosdb_mongo_database.example[0].name : var.existing_mongo_database_name
  throughput          = var.mongo_collection_throughput
  shard_key           = var.mongo_shard_key

  # Indexes
  dynamic "index" {
    for_each = var.mongo_indexes
    content {
      keys   = index.value.keys
      unique = index.value.unique
    }
  }

  autoscale_settings {
    max_throughput = var.mongo_collection_autoscale_max_throughput
  }
}

# Private Endpoint (if enabled)
resource "azurerm_private_endpoint" "cosmos_private_endpoint" {
  count               = var.enable_private_endpoint ? 1 : 0
  name                = "${var.cosmosdb_account_name}-private-endpoint"
  location            = var.location
  resource_group_name = var.resource_group_name
  subnet_id           = var.private_endpoint_subnet_id

  private_service_connection {
    name                           = "${var.cosmosdb_account_name}-private-service-connection"
    private_connection_resource_id = azurerm_cosmosdb_account.example.id
    subresource_names              = [var.kind == "MongoDB" ? "MongoDB" : "Sql"]
    is_manual_connection           = false
  }

  tags = merge(var.tags, {
    Name        = "${var.cosmosdb_account_name}-private-endpoint"
    Environment = "development"
  })
} 