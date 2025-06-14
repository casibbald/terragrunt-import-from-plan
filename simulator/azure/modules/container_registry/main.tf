# Azure Container Registry
resource "azurerm_container_registry" "example" {
  name                = var.registry_name
  resource_group_name = var.resource_group_name
  location            = var.location
  sku                 = var.sku
  admin_enabled       = var.admin_enabled

  # Public network access
  public_network_access_enabled = var.public_network_access_enabled

  # Network rule set (simplified for basic configuration)
  dynamic "network_rule_set" {
    for_each = var.network_rule_set_enabled ? [1] : []
    content {
      default_action = var.network_rule_default_action
    }
  }

  # Georeplications for Premium SKU
  dynamic "georeplications" {
    for_each = var.sku == "Premium" ? var.georeplications : []
    content {
      location                  = georeplications.value.location
      zone_redundancy_enabled   = georeplications.value.zone_redundancy_enabled
      regional_endpoint_enabled = georeplications.value.regional_endpoint_enabled
      tags                      = georeplications.value.tags
    }
  }

  # Identity
  identity {
    type = "SystemAssigned"
  }

  # Encryption for Premium SKU
  dynamic "encryption" {
    for_each = var.sku == "Premium" && var.encryption_enabled ? [1] : []
    content {
      enabled            = true
      key_vault_key_id   = var.encryption_key_vault_key_id
      identity_client_id = var.encryption_identity_client_id
    }
  }

  # Retention policy for Premium SKU
  dynamic "retention_policy" {
    for_each = var.sku == "Premium" && var.retention_policy_enabled ? [1] : []
    content {
      days    = var.retention_policy_days
      enabled = true
    }
  }

  # Trust policy for Premium SKU
  dynamic "trust_policy" {
    for_each = var.sku == "Premium" && var.trust_policy_enabled ? [1] : []
    content {
      enabled = true
    }
  }

  tags = merge(var.tags, {
    Name        = var.registry_name
    Environment = "development"
  })
}

# Private Endpoint (if enabled and Premium SKU)
resource "azurerm_private_endpoint" "acr_private_endpoint" {
  count               = var.enable_private_endpoint && var.sku == "Premium" ? 1 : 0
  name                = "${var.registry_name}-private-endpoint"
  location            = var.location
  resource_group_name = var.resource_group_name
  subnet_id           = var.private_endpoint_subnet_id

  private_service_connection {
    name                           = "${var.registry_name}-private-service-connection"
    private_connection_resource_id = azurerm_container_registry.example.id
    subresource_names              = ["registry"]
    is_manual_connection           = false
  }

  private_dns_zone_group {
    name                 = "private-dns-zone-group"
    private_dns_zone_ids = var.private_dns_zone_ids
  }

  tags = merge(var.tags, {
    Name        = "${var.registry_name}-private-endpoint"
    Environment = "development"
  })
}

# Role assignments for accessing the registry
resource "azurerm_role_assignment" "acr_pull" {
  count                = length(var.pull_role_assignment_principal_ids)
  scope                = azurerm_container_registry.example.id
  role_definition_name = "AcrPull"
  principal_id         = var.pull_role_assignment_principal_ids[count.index]
}

resource "azurerm_role_assignment" "acr_push" {
  count                = length(var.push_role_assignment_principal_ids)
  scope                = azurerm_container_registry.example.id
  role_definition_name = "AcrPush"
  principal_id         = var.push_role_assignment_principal_ids[count.index]
}

# Webhook (if enabled)
resource "azurerm_container_registry_webhook" "example" {
  count               = var.enable_webhook ? 1 : 0
  name                = var.webhook_name
  resource_group_name = var.resource_group_name
  registry_name       = azurerm_container_registry.example.name
  location            = var.location

  service_uri    = var.webhook_service_uri
  status         = "enabled"
  scope          = var.webhook_scope
  actions        = var.webhook_actions
  custom_headers = var.webhook_custom_headers

  tags = merge(var.tags, {
    Name        = var.webhook_name
    Environment = "development"
  })
}

# Scope Map (for token authentication with Premium SKU)
resource "azurerm_container_registry_scope_map" "example" {
  count                   = var.sku == "Premium" && var.enable_scope_map ? 1 : 0
  name                    = var.scope_map_name
  container_registry_name = azurerm_container_registry.example.name
  resource_group_name     = var.resource_group_name
  actions                 = var.scope_map_actions
}

# Token (for token authentication with Premium SKU)
resource "azurerm_container_registry_token" "example" {
  count                   = var.sku == "Premium" && var.enable_token ? 1 : 0
  name                    = var.token_name
  container_registry_name = azurerm_container_registry.example.name
  resource_group_name     = var.resource_group_name
  scope_map_id            = var.enable_scope_map ? azurerm_container_registry_scope_map.example[0].id : null
  enabled                 = true
} 