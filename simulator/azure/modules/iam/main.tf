# Azure Active Directory Application
resource "azuread_application" "example" {
  display_name = "example-application"
  owners       = [data.azuread_client_config.current.object_id]

  web {
    homepage_url = "https://example.com"
  }

  required_resource_access {
    resource_app_id = "00000003-0000-0000-c000-000000000000" # Microsoft Graph

    resource_access {
      id   = "e1fe6dd8-ba31-4d61-89e7-88639da4683d" # User.Read
      type = "Scope"
    }
  }
}

# Service Principal for the application
resource "azuread_service_principal" "example" {
  application_id               = azuread_application.example.application_id
  app_role_assignment_required = false
  owners                       = [data.azuread_client_config.current.object_id]
}

# Service Principal Password (Client Secret)
resource "azuread_service_principal_password" "example" {
  service_principal_id = azuread_service_principal.example.object_id
  display_name         = "example-client-secret"
}

# Custom Role Definition
resource "azurerm_role_definition" "example" {
  name  = "Example Custom Role"
  scope = "/subscriptions/${data.azurerm_client_config.current.subscription_id}"

  permissions {
    actions = [
      "Microsoft.Storage/storageAccounts/read",
      "Microsoft.Storage/storageAccounts/listKeys/action",
      "Microsoft.Compute/virtualMachines/read"
    ]
    not_actions = []
  }

  assignable_scopes = [
    "/subscriptions/${data.azurerm_client_config.current.subscription_id}"
  ]
}

# Role Assignment for Service Principal
resource "azurerm_role_assignment" "example" {
  scope                = "/subscriptions/${data.azurerm_client_config.current.subscription_id}"
  role_definition_name = "Reader"
  principal_id         = azuread_service_principal.example.object_id
}

# Custom Role Assignment
resource "azurerm_role_assignment" "custom" {
  scope              = "/subscriptions/${data.azurerm_client_config.current.subscription_id}"
  role_definition_id = azurerm_role_definition.example.role_definition_resource_id
  principal_id       = azuread_service_principal.example.object_id
}

# User-Assigned Managed Identity
resource "azurerm_user_assigned_identity" "example" {
  name                = "example-managed-identity"
  location            = var.location
  resource_group_name = var.resource_group_name
}

# Role Assignment for Managed Identity
resource "azurerm_role_assignment" "managed_identity" {
  scope                = "/subscriptions/${data.azurerm_client_config.current.subscription_id}"
  role_definition_name = "Contributor"
  principal_id         = azurerm_user_assigned_identity.example.principal_id
}

# Data sources
data "azurerm_client_config" "current" {}
data "azuread_client_config" "current" {} 