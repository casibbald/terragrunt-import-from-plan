# This file includes all AWS module declarations, wired with inputs and dependencies where needed

# IAM Module - foundational identity and access management
module "iam" {
  source = "./modules/iam"
}

# VPC Module - foundational networking
module "vpc" {
  source = "./modules/vpc"
  region = var.region
}

# S3 Module - object storage
module "s3" {
  source = "./modules/s3"
  region = var.region
}

# ECR Module - container registry
module "ecr" {
  source = "./modules/ecr"
}

# Lambda Module - serverless functions
module "lambda" {
  source = "./modules/lambda"
}

# RDS Module - relational database
module "rds" {
  source             = "./modules/rds"
  region             = var.region
  subnet_ids         = [module.vpc.public_subnet_id, module.vpc.private_subnet_id]
  security_group_ids = [module.vpc.security_group_id]
}

# KMS Module - key management service
module "kms" {
  source = "./modules/kms"
  region = var.region
}

# SNS Module - simple notification service
module "sns" {
  source = "./modules/sns"
  region = var.region
}

# Secrets Manager Module - secrets management
module "secrets_manager" {
  source = "./modules/secrets_manager"
  region = var.region
}

# CloudWatch Module - monitoring and logging
module "cloudwatch" {
  source     = "./modules/cloudwatch"
  region     = var.region
  log_group_name = "/aws/lambda/${module.lambda.function_name}"
}

# CloudTrail Module - audit logging
module "cloudtrail" {
  source      = "./modules/cloudtrail"
  region      = var.region
  bucket_name = module.s3.bucket_name
} 