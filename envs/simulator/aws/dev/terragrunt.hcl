locals {
  region     = "us-east-1"
  account_id = "123456789012"  # Example AWS account ID
}

terraform {
  source = "${get_repo_root()}/simulator/aws"
}

inputs = {
  region     = local.region
  account_id = local.account_id
} 