# RDS Subnet Group - required for RDS instance
resource "aws_db_subnet_group" "example" {
  name       = "example-db-subnet-group"
  subnet_ids = var.subnet_ids

  tags = {
    Name        = "example-db-subnet-group"
    Environment = "development"
  }
}

# RDS Parameter Group
resource "aws_db_parameter_group" "example" {
  family = "postgres15"
  name   = "example-pg-params"

  parameter {
    name  = "shared_preload_libraries"
    value = "pg_stat_statements"
  }

  tags = {
    Name        = "example-parameter-group"
    Environment = "development"
  }
}

# RDS Instance - equivalent to google_sql_database_instance
resource "aws_db_instance" "example" {
  identifier            = "example-postgres-db"
  engine                = "postgres"
  engine_version        = "15.4"
  instance_class        = "db.t3.micro"
  allocated_storage     = 20
  max_allocated_storage = 100
  storage_type          = "gp2"
  storage_encrypted     = true

  db_name  = "exampledb"
  username = "dbadmin"
  password = random_password.db_password.result

  vpc_security_group_ids = var.security_group_ids
  db_subnet_group_name   = aws_db_subnet_group.example.name
  parameter_group_name   = aws_db_parameter_group.example.name

  backup_retention_period = 7
  backup_window           = "03:00-04:00"
  maintenance_window      = "sun:04:00-sun:05:00"

  skip_final_snapshot = true
  deletion_protection = false

  enabled_cloudwatch_logs_exports = ["postgresql"]

  tags = {
    Name        = "example-postgres-db"
    Environment = "development"
  }
}

# Generate random password for database
resource "random_password" "db_password" {
  length  = 16
  special = true
}

# Store database password in AWS Secrets Manager
resource "aws_secretsmanager_secret" "db_password" {
  name        = "example-db-password"
  description = "Password for example PostgreSQL database"

  tags = {
    Name        = "example-db-secret"
    Environment = "development"
  }
}

resource "aws_secretsmanager_secret_version" "db_password" {
  secret_id = aws_secretsmanager_secret.db_password.id
  secret_string = jsonencode({
    username = aws_db_instance.example.username
    password = random_password.db_password.result
    endpoint = aws_db_instance.example.endpoint
    port     = aws_db_instance.example.port
    dbname   = aws_db_instance.example.db_name
  })
}

# RDS Enhanced Monitoring Role
resource "aws_iam_role" "rds_enhanced_monitoring" {
  name = "example-rds-enhanced-monitoring-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "monitoring.rds.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "rds_enhanced_monitoring" {
  role       = aws_iam_role.rds_enhanced_monitoring.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonRDSEnhancedMonitoringRole"
} 