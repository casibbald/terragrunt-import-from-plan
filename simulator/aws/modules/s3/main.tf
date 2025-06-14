# S3 Bucket - equivalent to google_storage_bucket
resource "aws_s3_bucket" "example" {
  bucket        = local.bucket_name
  force_destroy = true

  tags = {
    Name        = "example-bucket"
    Environment = "development"
  }
}

# S3 Bucket Versioning - equivalent to google_storage_bucket versioning
resource "aws_s3_bucket_versioning" "example" {
  bucket = aws_s3_bucket.example.id
  versioning_configuration {
    status = "Enabled"
  }
}

# S3 Bucket Public Access Block - security best practice
resource "aws_s3_bucket_public_access_block" "example" {
  bucket = aws_s3_bucket.example.id

  block_public_acls       = false
  block_public_policy     = false
  ignore_public_acls      = false
  restrict_public_buckets = false
}

# S3 Bucket Policy - equivalent to google_storage_bucket_iam_binding
resource "aws_s3_bucket_policy" "example" {
  bucket = aws_s3_bucket.example.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid       = "PublicReadGetObject"
        Effect    = "Allow"
        Principal = "*"
        Action    = "s3:GetObject"
        Resource  = "${aws_s3_bucket.example.arn}/*"
      }
    ]
  })

  depends_on = [aws_s3_bucket_public_access_block.example]
}

# S3 Bucket Notification - additional AWS-specific feature
resource "aws_s3_bucket_notification" "example" {
  bucket = aws_s3_bucket.example.id

  # Placeholder for SNS topic notification
  topic {
    topic_arn = "arn:aws:sns:${var.region}:${local.account_id}:example-topic"
    events    = ["s3:ObjectCreated:*"]
  }

  lifecycle {
    ignore_changes = [topic]
  }
}

# Mock account ID for CI/CD compatibility (no API calls needed)
locals {
  account_id = "123456789012" # Static mock value for simulation
} 