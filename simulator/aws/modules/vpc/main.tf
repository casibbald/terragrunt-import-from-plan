# Availability zones - use static values for CI/CD compatibility  
locals {
  # Static AZ names for CI/CD (works for any region pattern)
  availability_zones = ["${var.region}a", "${var.region}b", "${var.region}c"]
}

# VPC - Virtual Private Cloud
resource "aws_vpc" "example" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name        = "example-vpc"
    Environment = "development"
  }
}

# Internet Gateway
resource "aws_internet_gateway" "example" {
  vpc_id = aws_vpc.example.id

  tags = {
    Name        = "example-igw"
    Environment = "development"
  }
}

# Public Subnet
resource "aws_subnet" "public" {
  vpc_id                  = aws_vpc.example.id
  cidr_block              = "10.0.1.0/24"
  availability_zone       = local.availability_zones[0]
  map_public_ip_on_launch = true

  tags = {
    Name        = "example-public-subnet"
    Environment = "development"
    Type        = "public"
  }
}

# Private Subnet
resource "aws_subnet" "private" {
  vpc_id            = aws_vpc.example.id
  cidr_block        = "10.0.2.0/24"
  availability_zone = local.availability_zones[1 % length(local.availability_zones)]

  tags = {
    Name        = "example-private-subnet"
    Environment = "development"
    Type        = "private"
  }
}

# Route Table for Public Subnet
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.example.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.example.id
  }

  tags = {
    Name        = "example-public-rt"
    Environment = "development"
  }
}

# Route Table Association for Public Subnet
resource "aws_route_table_association" "public" {
  subnet_id      = aws_subnet.public.id
  route_table_id = aws_route_table.public.id
}

# Security Group
resource "aws_security_group" "example" {
  name_prefix = "example-sg"
  vpc_id      = aws_vpc.example.id

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "example-security-group"
    Environment = "development"
  }
} 