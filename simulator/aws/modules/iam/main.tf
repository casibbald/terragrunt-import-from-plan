# IAM Role - equivalent to google_service_account
resource "aws_iam_role" "example" {
  name = "example-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      }
    ]
  })

  tags = {
    Name        = "example-role"
    Environment = "development"
  }
}

# IAM Policy - custom policy for the role
resource "aws_iam_policy" "example" {
  name        = "example-policy"
  description = "Example policy for simulator"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject"
        ]
        Resource = "*"
      }
    ]
  })
}

# IAM Role Policy Attachment
resource "aws_iam_role_policy_attachment" "example" {
  role       = aws_iam_role.example.name
  policy_arn = aws_iam_policy.example.arn
}

# IAM Instance Profile - for EC2 instances
resource "aws_iam_instance_profile" "example" {
  name = "example-instance-profile"
  role = aws_iam_role.example.name
}

# IAM User - for programmatic access
resource "aws_iam_user" "example" {
  name = "example-user"

  tags = {
    Name        = "example-user"
    Environment = "development"
  }
}

# IAM Access Key for the user
resource "aws_iam_access_key" "example" {
  user = aws_iam_user.example.name
} 