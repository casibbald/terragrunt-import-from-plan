locals {
  subscription_id     = "12345678-1234-1234-1234-123456789012"
  resource_group_name = "rg-terraform-simulator-dev"
  location           = "East US"
}

terraform {
  source = "${get_repo_root()}/simulator/azure"
}

inputs = {
  subscription_id     = local.subscription_id
  resource_group_name = local.resource_group_name
  location           = local.location
} 