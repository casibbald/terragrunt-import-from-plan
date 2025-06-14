# Azure availability zones - define as locals for safe indexing
# Note: Use safe indexing pattern: zones[index % length(zones)] to handle regions with varying zone counts
locals {
  # Azure zones are typically 1, 2, 3 where available
  # This approach allows for safe indexing across different regions
  available_zones = ["1", "2", "3"]
  zone_count      = length(local.available_zones)
}

# Resource Group
resource "azurerm_resource_group" "example" {
  name     = var.resource_group_name
  location = var.location

  tags = merge(var.tags, {
    Name        = var.resource_group_name
    Environment = var.environment
  })
}

# Virtual Network
resource "azurerm_virtual_network" "example" {
  name                = var.vnet_name
  address_space       = var.vnet_address_space
  location            = azurerm_resource_group.example.location
  resource_group_name = azurerm_resource_group.example.name

  tags = merge(var.tags, {
    Name        = var.vnet_name
    Environment = var.environment
  })
}

# Public Subnet
resource "azurerm_subnet" "public" {
  name                 = "${var.vnet_name}-public-subnet"
  resource_group_name  = azurerm_resource_group.example.name
  virtual_network_name = azurerm_virtual_network.example.name
  address_prefixes     = var.public_subnet_prefix
}

# Private Subnet
resource "azurerm_subnet" "private" {
  name                 = "${var.vnet_name}-private-subnet"
  resource_group_name  = azurerm_resource_group.example.name
  virtual_network_name = azurerm_virtual_network.example.name
  address_prefixes     = var.private_subnet_prefix
}

# Network Security Group for Public Subnet
resource "azurerm_network_security_group" "public" {
  name                = "${var.vnet_name}-public-nsg"
  location            = azurerm_resource_group.example.location
  resource_group_name = azurerm_resource_group.example.name

  security_rule {
    name                       = "HTTP"
    priority                   = 1001
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "80"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  security_rule {
    name                       = "HTTPS"
    priority                   = 1002
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "443"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  tags = merge(var.tags, {
    Name        = "${var.vnet_name}-public-nsg"
    Environment = var.environment
  })
}

# Network Security Group for Private Subnet
resource "azurerm_network_security_group" "private" {
  name                = "${var.vnet_name}-private-nsg"
  location            = azurerm_resource_group.example.location
  resource_group_name = azurerm_resource_group.example.name

  tags = merge(var.tags, {
    Name        = "${var.vnet_name}-private-nsg"
    Environment = var.environment
  })
}

# Associate Network Security Group to Public Subnet
resource "azurerm_subnet_network_security_group_association" "public" {
  subnet_id                 = azurerm_subnet.public.id
  network_security_group_id = azurerm_network_security_group.public.id
}

# Associate Network Security Group to Private Subnet
resource "azurerm_subnet_network_security_group_association" "private" {
  subnet_id                 = azurerm_subnet.private.id
  network_security_group_id = azurerm_network_security_group.private.id
}

# Public IP for NAT Gateway (for private subnet internet access)
resource "azurerm_public_ip" "nat_gateway" {
  name                = "${var.vnet_name}-nat-gateway-pip"
  location            = azurerm_resource_group.example.location
  resource_group_name = azurerm_resource_group.example.name
  allocation_method   = "Static"
  sku                 = "Standard"

  tags = merge(var.tags, {
    Name        = "${var.vnet_name}-nat-gateway-pip"
    Environment = var.environment
  })
}

# NAT Gateway for Private Subnet
resource "azurerm_nat_gateway" "example" {
  name                    = "${var.vnet_name}-nat-gateway"
  location                = azurerm_resource_group.example.location
  resource_group_name     = azurerm_resource_group.example.name
  sku_name                = "Standard"
  idle_timeout_in_minutes = 10

  tags = merge(var.tags, {
    Name        = "${var.vnet_name}-nat-gateway"
    Environment = var.environment
  })
}

# Associate Public IP to NAT Gateway
resource "azurerm_nat_gateway_public_ip_association" "example" {
  nat_gateway_id       = azurerm_nat_gateway.example.id
  public_ip_address_id = azurerm_public_ip.nat_gateway.id
}

# Associate NAT Gateway to Private Subnet
resource "azurerm_subnet_nat_gateway_association" "example" {
  subnet_id      = azurerm_subnet.private.id
  nat_gateway_id = azurerm_nat_gateway.example.id
}

# Example of safe availability zone usage for future resources:
# zones = [local.available_zones[0 % local.zone_count]]
# zones = [local.available_zones[1 % local.zone_count]]
# or for single zone assignment:
# availability_zone = local.available_zones[0 % local.zone_count]
# availability_zone = local.available_zones[1 % local.zone_count] 