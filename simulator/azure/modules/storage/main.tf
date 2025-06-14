# Storage Account
resource "azurerm_storage_account" "example" {
  name                     = var.storage_account_name
  resource_group_name      = var.resource_group_name
  location                 = var.location
  account_tier             = var.account_tier
  account_replication_type = var.replication_type
  account_kind             = "StorageV2"

  # Security settings
  enable_https_traffic_only       = true
  min_tls_version                 = "TLS1_2"
  allow_nested_items_to_be_public = false

  # Access tier for blob storage
  access_tier = var.access_tier

  # Network rules
  network_rules {
    default_action = "Allow"
    ip_rules       = var.allowed_ip_ranges
    bypass         = ["AzureServices"]
  }

  # Blob properties
  blob_properties {
    delete_retention_policy {
      days = var.blob_retention_days
    }

    container_delete_retention_policy {
      days = var.container_retention_days
    }

    versioning_enabled = var.versioning_enabled

    cors_rule {
      allowed_headers    = ["*"]
      allowed_methods    = ["GET", "HEAD", "POST", "PUT", "DELETE"]
      allowed_origins    = ["*"]
      exposed_headers    = ["*"]
      max_age_in_seconds = 3600
    }
  }

  tags = merge(var.tags, {
    Name        = var.storage_account_name
    Environment = "development"
  })
}

# Storage Container (equivalent to S3 bucket)
resource "azurerm_storage_container" "example" {
  name                  = var.container_name
  storage_account_name  = azurerm_storage_account.example.name
  container_access_type = var.container_access_type
}

# Storage Container for logs
resource "azurerm_storage_container" "logs" {
  name                  = "${var.container_name}-logs"
  storage_account_name  = azurerm_storage_account.example.name
  container_access_type = "private"
}

# Storage Container for backups
resource "azurerm_storage_container" "backups" {
  name                  = "${var.container_name}-backups"
  storage_account_name  = azurerm_storage_account.example.name
  container_access_type = "private"
}

# Example blob upload
resource "azurerm_storage_blob" "example" {
  name                   = "example.txt"
  storage_account_name   = azurerm_storage_account.example.name
  storage_container_name = azurerm_storage_container.example.name
  type                   = "Block"
  content_type           = "text/plain"
  source_content         = "Hello, Azure Blob Storage!"
}

# Storage Account Static Website (if enabled)
resource "azurerm_storage_account_static_website" "example" {
  count              = var.enable_static_website ? 1 : 0
  storage_account_id = azurerm_storage_account.example.id
  index_document     = "index.html"
  error_404_document = "404.html"
}

# Storage Management Policy
resource "azurerm_storage_management_policy" "example" {
  storage_account_id = azurerm_storage_account.example.id

  rule {
    name    = "lifecycle-rule"
    enabled = true

    filters {
      prefix_match = ["documents/"]
      blob_types   = ["blockBlob"]
    }

    actions {
      base_blob {
        tier_to_cool_after_days_since_modification_greater_than    = 30
        tier_to_archive_after_days_since_modification_greater_than = 90
        delete_after_days_since_modification_greater_than          = 365
      }

      snapshot {
        delete_after_days_since_creation_greater_than = 30
      }
    }
  }
} 