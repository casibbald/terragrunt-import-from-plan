# CloudWatch Log Group - equivalent to google_logging_project_sink
resource "aws_cloudwatch_log_group" "example" {
  name              = var.log_group_name
  retention_in_days = 7

  tags = {
    Name        = "example-log-group"
    Environment = "development"
  }
}

# CloudWatch Metric Alarm
resource "aws_cloudwatch_metric_alarm" "example" {
  alarm_name          = "example-alarm"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "CPUUtilization"
  namespace           = "AWS/EC2"
  period              = "120"
  statistic           = "Average"
  threshold           = "80"
  alarm_description   = "This metric monitors ec2 cpu utilization"

  tags = {
    Name        = "example-alarm"
    Environment = "development"
  }
}

# CloudWatch Dashboard
resource "aws_cloudwatch_dashboard" "example" {
  dashboard_name = "example-dashboard"

  dashboard_body = jsonencode({
    widgets = [
      {
        type   = "metric"
        x      = 0
        y      = 0
        width  = 12
        height = 6

        properties = {
          metrics = [
            ["AWS/EC2", "CPUUtilization"]
          ]
          view    = "timeSeries"
          stacked = false
          region  = var.region
          title   = "EC2 Instance CPU"
          period  = 300
        }
      }
    ]
  })
} 