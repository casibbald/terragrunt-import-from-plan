# SQL Server
resource "azurerm_mssql_server" "example" {
  name                         = var.sql_server_name
  resource_group_name          = var.resource_group_name
  location                     = var.location
  version                      = var.sql_server_version
  administrator_login          = var.administrator_login
  administrator_login_password = var.administrator_login_password

  # Security settings
  minimum_tls_version = var.minimum_tls_version

  # Azure AD authentication
  azuread_administrator {
    login_username = var.azuread_admin_login
    object_id      = var.azuread_admin_object_id
  }

  tags = merge(var.tags, {
    Name        = var.sql_server_name
    Environment = "development"
  })
}

# SQL Database
resource "azurerm_mssql_database" "example" {
  name           = var.database_name
  server_id      = azurerm_mssql_server.example.id
  collation      = var.database_collation
  license_type   = var.license_type
  max_size_gb    = var.max_size_gb
  sku_name       = var.database_sku_name
  zone_redundant = var.zone_redundant

  # Backup settings
  short_term_retention_policy {
    retention_days = var.backup_retention_days
  }

  long_term_retention_policy {
    weekly_retention  = var.weekly_retention
    monthly_retention = var.monthly_retention
    yearly_retention  = var.yearly_retention
    week_of_year      = var.week_of_year
  }

  tags = merge(var.tags, {
    Name        = var.database_name
    Environment = "development"
  })
}

# Firewall rule to allow Azure services
resource "azurerm_mssql_firewall_rule" "azure_services" {
  name             = "AllowAzureServices"
  server_id        = azurerm_mssql_server.example.id
  start_ip_address = "0.0.0.0"
  end_ip_address   = "0.0.0.0"
}

# Additional firewall rules for specific IP ranges
resource "azurerm_mssql_firewall_rule" "allowed_ips" {
  count            = length(var.allowed_ip_ranges)
  name             = "AllowedIP${count.index + 1}"
  server_id        = azurerm_mssql_server.example.id
  start_ip_address = split("-", var.allowed_ip_ranges[count.index])[0]
  end_ip_address   = length(split("-", var.allowed_ip_ranges[count.index])) > 1 ? split("-", var.allowed_ip_ranges[count.index])[1] : split("-", var.allowed_ip_ranges[count.index])[0]
}

# SQL Database Extended Auditing Policy
resource "azurerm_mssql_database_extended_auditing_policy" "example" {
  count                                   = var.enable_auditing ? 1 : 0
  database_id                             = azurerm_mssql_database.example.id
  storage_endpoint                        = var.audit_storage_endpoint
  storage_account_access_key              = var.audit_storage_account_access_key
  storage_account_access_key_is_secondary = false
  retention_in_days                       = var.audit_retention_in_days
}

# SQL Server Extended Auditing Policy
resource "azurerm_mssql_server_extended_auditing_policy" "example" {
  count                                   = var.enable_auditing ? 1 : 0
  server_id                               = azurerm_mssql_server.example.id
  storage_endpoint                        = var.audit_storage_endpoint
  storage_account_access_key              = var.audit_storage_account_access_key
  storage_account_access_key_is_secondary = false
  retention_in_days                       = var.audit_retention_in_days
}

# Note: Transparent Data Encryption is enabled by default on Azure SQL Database

# Vulnerability Assessment
resource "azurerm_mssql_server_vulnerability_assessment" "example" {
  count                           = var.enable_vulnerability_assessment ? 1 : 0
  server_security_alert_policy_id = azurerm_mssql_server_security_alert_policy.example[0].id
  storage_container_path          = var.va_storage_container_path
  storage_account_access_key      = var.va_storage_account_access_key

  recurring_scans {
    enabled                   = true
    email_subscription_admins = true
    emails                    = var.va_notification_emails
  }
}

# Security Alert Policy
resource "azurerm_mssql_server_security_alert_policy" "example" {
  count                      = var.enable_vulnerability_assessment ? 1 : 0
  resource_group_name        = var.resource_group_name
  server_name                = azurerm_mssql_server.example.name
  state                      = "Enabled"
  storage_endpoint           = var.security_alert_storage_endpoint
  storage_account_access_key = var.security_alert_storage_account_access_key
  retention_days             = var.security_alert_retention_days

  disabled_alerts      = []
  email_account_admins = true
  email_addresses      = var.security_alert_email_addresses
}

# Private Endpoint (if enabled)
resource "azurerm_private_endpoint" "sql_private_endpoint" {
  count               = var.enable_private_endpoint ? 1 : 0
  name                = "${var.sql_server_name}-private-endpoint"
  location            = var.location
  resource_group_name = var.resource_group_name
  subnet_id           = var.private_endpoint_subnet_id

  private_service_connection {
    name                           = "${var.sql_server_name}-private-service-connection"
    private_connection_resource_id = azurerm_mssql_server.example.id
    subresource_names              = ["sqlServer"]
    is_manual_connection           = false
  }

  tags = merge(var.tags, {
    Name        = "${var.sql_server_name}-private-endpoint"
    Environment = "development"
  })
} 