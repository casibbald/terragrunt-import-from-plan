# Secrets Manager Secret
resource "aws_secretsmanager_secret" "example" {
  name        = "example-secret"
  description = "Example secret for simulation"

  tags = {
    Name        = "example-secret"
    Environment = "development"
  }
}

# Secrets Manager Secret Version
resource "aws_secretsmanager_secret_version" "example" {
  secret_id = aws_secretsmanager_secret.example.id
  secret_string = jsonencode({
    username = "admin"
    password = "example-password"
  })
} 