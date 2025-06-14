# KMS Key - equivalent to google_kms_crypto_key
resource "aws_kms_key" "example" {
  description             = "Example KMS key for simulation"
  deletion_window_in_days = 7
  enable_key_rotation     = true

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "Enable IAM User Permissions"
        Effect = "Allow"
        Principal = {
          AWS = "arn:aws:iam::${data.aws_caller_identity.current.account_id}:root"
        }
        Action   = "kms:*"
        Resource = "*"
      }
    ]
  })

  tags = {
    Name        = "example-kms-key"
    Environment = "development"
  }
}

# KMS Key Alias
resource "aws_kms_alias" "example" {
  name          = "alias/example-key"
  target_key_id = aws_kms_key.example.key_id
}

# Get current AWS account ID
data "aws_caller_identity" "current" {} 