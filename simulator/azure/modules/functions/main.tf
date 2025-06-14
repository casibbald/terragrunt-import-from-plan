# Storage Account for Function App
resource "azurerm_storage_account" "functions" {
  name                     = var.function_storage_account_name
  resource_group_name      = var.resource_group_name
  location                 = var.location
  account_tier             = "Standard"
  account_replication_type = "LRS"

  tags = merge(var.tags, {
    Name        = var.function_storage_account_name
    Environment = "development"
    Purpose     = "functions"
  })
}

# App Service Plan for Function App
resource "azurerm_service_plan" "functions" {
  name                = var.service_plan_name
  location            = var.location
  resource_group_name = var.resource_group_name
  os_type             = var.os_type
  sku_name            = var.sku_name

  tags = merge(var.tags, {
    Name        = var.service_plan_name
    Environment = "development"
  })
}

# Function App
resource "azurerm_linux_function_app" "example" {
  count               = var.os_type == "Linux" ? 1 : 0
  name                = var.function_app_name
  location            = var.location
  resource_group_name = var.resource_group_name
  service_plan_id     = azurerm_service_plan.functions.id

  storage_account_name       = azurerm_storage_account.functions.name
  storage_account_access_key = azurerm_storage_account.functions.primary_access_key

  # Application settings
  app_settings = merge(var.app_settings, {
    "WEBSITE_RUN_FROM_PACKAGE" = "1"
    "FUNCTIONS_WORKER_RUNTIME" = var.runtime
  })

  site_config {
    always_on = var.always_on

    application_stack {
      node_version   = var.runtime == "node" ? var.runtime_version : null
      python_version = var.runtime == "python" ? var.runtime_version : null
      dotnet_version = var.runtime == "dotnet" ? var.runtime_version : null
      java_version   = var.runtime == "java" ? var.runtime_version : null
    }

    # CORS settings
    cors {
      allowed_origins     = var.cors_allowed_origins
      support_credentials = var.cors_support_credentials
    }
  }

  # Identity
  identity {
    type = "SystemAssigned"
  }

  tags = merge(var.tags, {
    Name        = var.function_app_name
    Environment = "development"
  })
}

# Windows Function App (alternative)
resource "azurerm_windows_function_app" "example" {
  count               = var.os_type == "Windows" ? 1 : 0
  name                = var.function_app_name
  location            = var.location
  resource_group_name = var.resource_group_name
  service_plan_id     = azurerm_service_plan.functions.id

  storage_account_name       = azurerm_storage_account.functions.name
  storage_account_access_key = azurerm_storage_account.functions.primary_access_key

  # Application settings
  app_settings = merge(var.app_settings, {
    "WEBSITE_RUN_FROM_PACKAGE" = "1"
    "FUNCTIONS_WORKER_RUNTIME" = var.runtime
  })

  site_config {
    always_on = var.always_on

    application_stack {
      node_version   = var.runtime == "node" ? var.runtime_version : null
      dotnet_version = var.runtime == "dotnet" ? var.runtime_version : null
      java_version   = var.runtime == "java" ? var.runtime_version : null
    }

    # CORS settings
    cors {
      allowed_origins     = var.cors_allowed_origins
      support_credentials = var.cors_support_credentials
    }
  }

  # Identity
  identity {
    type = "SystemAssigned"
  }

  tags = merge(var.tags, {
    Name        = var.function_app_name
    Environment = "development"
  })
}

# Application Insights for monitoring
resource "azurerm_application_insights" "functions" {
  count               = var.enable_application_insights ? 1 : 0
  name                = "${var.function_app_name}-insights"
  location            = var.location
  resource_group_name = var.resource_group_name
  application_type    = "web"

  tags = merge(var.tags, {
    Name        = "${var.function_app_name}-insights"
    Environment = "development"
  })
}

# Example Function (HTTP Trigger)
resource "azurerm_function_app_function" "http_example" {
  count           = var.create_example_function ? 1 : 0
  name            = "HttpExample"
  function_app_id = var.os_type == "Linux" ? (length(azurerm_linux_function_app.example) > 0 ? azurerm_linux_function_app.example[0].id : null) : (length(azurerm_windows_function_app.example) > 0 ? azurerm_windows_function_app.example[0].id : null)
  language        = var.runtime == "node" ? "Javascript" : title(var.runtime)

  file {
    name    = "index.js"
    content = <<-EOT
      module.exports = async function (context, req) {
          context.log('JavaScript HTTP trigger function processed a request.');
          
          const name = (req.query.name || (req.body && req.body.name));
          const responseMessage = name
              ? "Hello, " + name + ". This HTTP triggered function executed successfully."
              : "This HTTP triggered function executed successfully. Pass a name in the query string or in the request body for a personalized response.";
          
          context.res = {
              // status: 200, /* Defaults to 200 */
              body: responseMessage
          };
      }
    EOT
  }

  config_json = jsonencode({
    "bindings" = [
      {
        "authLevel" = "function"
        "type"      = "httpTrigger"
        "direction" = "in"
        "name"      = "req"
        "methods"   = ["get", "post"]
      },
      {
        "type"      = "http"
        "direction" = "out"
        "name"      = "res"
      }
    ]
  })
} 