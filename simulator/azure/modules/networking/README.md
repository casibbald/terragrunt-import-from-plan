# Azure Networking Module

This module creates a complete Azure networking infrastructure including VNet, subnets, Network Security Groups, and NAT Gateway.

## Features

- **Virtual Network (VNet)** with configurable address space
- **Public and Private Subnets** with proper network segmentation
- **Network Security Groups** with basic HTTP/HTTPS rules
- **NAT Gateway** for private subnet internet access
- **Safe Availability Zone Handling** for multi-AZ deployments

## Safe Availability Zone Pattern

This module implements a safe availability zone indexing pattern to handle regions with varying numbers of availability zones:

```hcl
# Safe indexing pattern - prevents index out of bounds errors
availability_zone = data.azurerm_availability_zones.available.zones[0 % length(data.azurerm_availability_zones.available.zones)]
availability_zone = data.azurerm_availability_zones.available.zones[1 % length(data.azurerm_availability_zones.available.zones)]
```

**How it works:**
- **Regions with 3+ zones**: Uses zones 1, 2, 3 as expected
- **Regions with 2 zones**: Uses zones 1, 2, then wraps back to 1
- **Regions with 1 zone**: Uses zone 1 for all resources (prevents failures)

## Usage

```hcl
module "networking" {
  source = "./modules/networking"
  
  location               = "East US"
  resource_group_name    = "my-app-resources"
  vnet_name             = "my-app-vnet"
  vnet_address_space    = ["10.0.0.0/16"]
  public_subnet_prefix  = ["10.0.1.0/24"]
  private_subnet_prefix = ["10.0.2.0/24"]
  environment           = "production"
  
  tags = {
    Project = "MyApp"
    Owner   = "DevOps Team"
  }
}
```

## Example with VM Deployment Across Zones

```hcl
# Deploy VMs across zones safely
resource "azurerm_virtual_machine" "web" {
  count = 2
  
  name                = "web-vm-${count.index + 1}"
  location            = module.networking.resource_group_location
  resource_group_name = module.networking.resource_group_name
  
  # Safe zone assignment
  zones = [data.azurerm_availability_zones.available.zones[count.index % length(data.azurerm_availability_zones.available.zones)]]
  
  # ... other VM configuration
}
```

## Inputs

| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:--------:|
| location | The Azure region where resources will be created | `string` | `"East US"` | no |
| resource_group_name | The name of the resource group | `string` | `"example-resources"` | no |
| vnet_name | The name of the virtual network | `string` | `"example-vnet"` | no |
| vnet_address_space | The address space for the virtual network | `list(string)` | `["10.0.0.0/16"]` | no |
| public_subnet_prefix | The address prefix for the public subnet | `list(string)` | `["10.0.1.0/24"]` | no |
| private_subnet_prefix | The address prefix for the private subnet | `list(string)` | `["10.0.2.0/24"]` | no |
| environment | The environment tag | `string` | `"development"` | no |
| tags | Additional tags to apply to resources | `map(string)` | `{}` | no |

## Outputs

| Name | Description |
|------|-------------|
| resource_group_name | The name of the resource group |
| resource_group_id | The ID of the resource group |
| vnet_name | The name of the virtual network |
| vnet_id | The ID of the virtual network |
| public_subnet_id | The ID of the public subnet |
| private_subnet_id | The ID of the private subnet |
| available_zones | Available availability zones in the region |
| zone_count | Number of available zones (useful for safe indexing) |

## Multi-Cloud Pattern

This module follows the same pattern as the AWS VPC module:

**AWS Pattern:**
```hcl
availability_zone = data.aws_availability_zones.available.names[1 % length(data.aws_availability_zones.available.names)]
```

**Azure Pattern:**
```hcl
availability_zone = data.azurerm_availability_zones.available.zones[1 % length(data.azurerm_availability_zones.available.zones)]
```

**GCP Pattern (future):**
```hcl
zone = data.google_compute_zones.available.names[1 % length(data.google_compute_zones.available.names)]
```

This consistent approach ensures infrastructure code works reliably across all cloud providers and regions. 