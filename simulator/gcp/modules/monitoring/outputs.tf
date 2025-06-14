
// outputs.tf

output "alert_policy_id" {
  value       = google_monitoring_alert_policy.basic_policy.name
  description = "Name of the alert policy"
}

