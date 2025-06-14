# SNS Topic - equivalent to google_pubsub_topic
resource "aws_sns_topic" "example" {
  name = "example-topic"

  tags = {
    Name        = "example-topic"
    Environment = "development"
  }
}

# SNS Topic Policy
resource "aws_sns_topic_policy" "example" {
  arn = aws_sns_topic.example.arn

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          AWS = "arn:aws:iam::${data.aws_caller_identity.current.account_id}:root"
        }
        Action = [
          "SNS:Publish",
          "SNS:Subscribe"
        ]
        Resource = aws_sns_topic.example.arn
      }
    ]
  })
}

# SNS Subscription - equivalent to google_pubsub_subscription
resource "aws_sns_topic_subscription" "example" {
  topic_arn = aws_sns_topic.example.arn
  protocol  = "email"
  endpoint  = "example@example.com"

  depends_on = [aws_sns_topic.example]
}

# SQS Queue for SNS (dead letter queue)
resource "aws_sqs_queue" "example_dlq" {
  name = "example-dlq"

  tags = {
    Name        = "example-dlq"
    Environment = "development"
  }
}

# SQS Queue for SNS messages
resource "aws_sqs_queue" "example" {
  name = "example-queue"

  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.example_dlq.arn
    maxReceiveCount     = 3
  })

  tags = {
    Name        = "example-queue"
    Environment = "development"
  }
}

# SNS Topic Subscription to SQS
resource "aws_sns_topic_subscription" "sqs" {
  topic_arn = aws_sns_topic.example.arn
  protocol  = "sqs"
  endpoint  = aws_sqs_queue.example.arn
}

# SQS Queue Policy to allow SNS to send messages
resource "aws_sqs_queue_policy" "example" {
  queue_url = aws_sqs_queue.example.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          Service = "sns.amazonaws.com"
        }
        Action = "sqs:SendMessage"
        Resource = aws_sqs_queue.example.arn
        Condition = {
          ArnEquals = {
            "aws:SourceArn" = aws_sns_topic.example.arn
          }
        }
      }
    ]
  })
}

# Get current AWS account ID
data "aws_caller_identity" "current" {} 