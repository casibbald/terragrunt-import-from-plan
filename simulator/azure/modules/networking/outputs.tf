output "resource_group_name" {
  description = "The name of the resource group"
  value       = azurerm_resource_group.example.name
}

output "resource_group_id" {
  description = "The ID of the resource group"
  value       = azurerm_resource_group.example.id
}

output "vnet_name" {
  description = "The name of the virtual network"
  value       = azurerm_virtual_network.example.name
}

output "vnet_id" {
  description = "The ID of the virtual network"
  value       = azurerm_virtual_network.example.id
}

output "vnet_address_space" {
  description = "The address space of the virtual network"
  value       = azurerm_virtual_network.example.address_space
}

output "public_subnet_name" {
  description = "The name of the public subnet"
  value       = azurerm_subnet.public.name
}

output "public_subnet_id" {
  description = "The ID of the public subnet"
  value       = azurerm_subnet.public.id
}

output "private_subnet_name" {
  description = "The name of the private subnet"
  value       = azurerm_subnet.private.name
}

output "private_subnet_id" {
  description = "The ID of the private subnet"
  value       = azurerm_subnet.private.id
}

output "public_nsg_id" {
  description = "The ID of the public network security group"
  value       = azurerm_network_security_group.public.id
}

output "private_nsg_id" {
  description = "The ID of the private network security group"
  value       = azurerm_network_security_group.private.id
}

output "nat_gateway_id" {
  description = "The ID of the NAT gateway"
  value       = azurerm_nat_gateway.example.id
}

output "nat_gateway_public_ip" {
  description = "The public IP address of the NAT gateway"
  value       = azurerm_public_ip.nat_gateway.ip_address
}

output "available_zones" {
  description = "Available availability zones in the region"
  value       = local.available_zones
}

output "zone_count" {
  description = "Number of available zones (useful for safe indexing)"
  value       = local.zone_count
} 