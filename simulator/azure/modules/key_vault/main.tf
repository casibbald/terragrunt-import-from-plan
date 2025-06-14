# Key Vault
resource "azurerm_key_vault" "example" {
  name                = var.key_vault_name
  location            = var.location
  resource_group_name = var.resource_group_name
  tenant_id           = data.azurerm_client_config.current.tenant_id
  sku_name            = var.sku_name

  # Enable features
  enabled_for_disk_encryption     = var.enabled_for_disk_encryption
  enabled_for_deployment          = var.enabled_for_deployment
  enabled_for_template_deployment = var.enabled_for_template_deployment
  purge_protection_enabled        = var.purge_protection_enabled
  soft_delete_retention_days      = var.soft_delete_retention_days

  # Network access rules
  network_acls {
    default_action = var.default_action
    bypass         = "AzureServices"
    ip_rules       = var.allowed_ip_ranges
  }

  tags = merge(var.tags, {
    Name        = var.key_vault_name
    Environment = "development"
  })
}

# Access Policy for Current User/Service Principal
resource "azurerm_key_vault_access_policy" "current" {
  key_vault_id = azurerm_key_vault.example.id
  tenant_id    = data.azurerm_client_config.current.tenant_id
  object_id    = data.azurerm_client_config.current.object_id

  key_permissions = [
    "Backup", "Create", "Decrypt", "Delete", "Encrypt", "Get", "Import",
    "List", "Purge", "Recover", "Restore", "Sign", "UnwrapKey", "Update",
    "Verify", "WrapKey", "Release", "Rotate", "GetRotationPolicy", "SetRotationPolicy"
  ]

  secret_permissions = [
    "Backup", "Delete", "Get", "List", "Purge", "Recover", "Restore", "Set"
  ]

  certificate_permissions = [
    "Backup", "Create", "Delete", "DeleteIssuers", "Get", "GetIssuers",
    "Import", "List", "ListIssuers", "ManageContacts", "ManageIssuers",
    "Purge", "Recover", "Restore", "SetIssuers", "Update"
  ]
}

# Example Key
resource "azurerm_key_vault_key" "example" {
  name         = "example-key"
  key_vault_id = azurerm_key_vault.example.id
  key_type     = "RSA"
  key_size     = 2048

  key_opts = [
    "decrypt", "encrypt", "sign", "unwrapKey", "verify", "wrapKey"
  ]

  rotation_policy {
    automatic {
      time_before_expiry = "P30D"
    }

    expire_after         = "P90D"
    notify_before_expiry = "P29D"
  }

  depends_on = [azurerm_key_vault_access_policy.current]
}

# Example Secret
resource "azurerm_key_vault_secret" "example" {
  name         = "example-secret"
  value        = var.secret_value
  key_vault_id = azurerm_key_vault.example.id

  expiration_date = timeadd(timestamp(), "8760h") # 1 year from now

  depends_on = [azurerm_key_vault_access_policy.current]
}

# Example Certificate
resource "azurerm_key_vault_certificate" "example" {
  name         = "example-certificate"
  key_vault_id = azurerm_key_vault.example.id

  certificate_policy {
    issuer_parameters {
      name = "Self"
    }

    key_properties {
      exportable = true
      key_size   = 2048
      key_type   = "RSA"
      reuse_key  = true
    }

    lifetime_action {
      action {
        action_type = "AutoRenew"
      }

      trigger {
        days_before_expiry = 30
      }
    }

    secret_properties {
      content_type = "application/x-pkcs12"
    }

    x509_certificate_properties {
      extended_key_usage = ["1.3.6.1.5.5.7.3.1"]

      key_usage = [
        "cRLSign", "dataEncipherment", "digitalSignature", "keyAgreement",
        "keyCertSign", "keyEncipherment"
      ]

      subject            = "CN=example.com"
      validity_in_months = 12

      subject_alternative_names {
        dns_names = ["example.com", "www.example.com"]
      }
    }
  }

  depends_on = [azurerm_key_vault_access_policy.current]
}

# Diagnostic Settings for Key Vault
resource "azurerm_monitor_diagnostic_setting" "key_vault_logs" {
  count              = var.enable_logging ? 1 : 0
  name               = "key-vault-logs"
  target_resource_id = azurerm_key_vault.example.id
  storage_account_id = var.log_storage_account_id

  enabled_log {
    category = "AuditEvent"
  }

  metric {
    category = "AllMetrics"
    enabled  = true
  }
}

# Data source
data "azurerm_client_config" "current" {} 