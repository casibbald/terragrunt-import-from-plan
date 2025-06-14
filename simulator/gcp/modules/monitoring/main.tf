// main.tf

resource "google_monitoring_notification_channel" "email" {
  display_name = "Simulation Alert Channel"
  type         = "email"
  project      = var.project_id
  labels = {
    email_address = var.alert_email
  }
}

resource "google_monitoring_alert_policy" "basic_policy" {
  display_name = "High CPU Usage"
  combiner     = "OR"
  project      = var.project_id

  conditions {
    display_name = "VM Instance CPU Usage"

    condition_threshold {
      filter          = "metric.type=\"compute.googleapis.com/instance/cpu/utilization\""
      comparison      = "COMPARISON_GT"
      threshold_value = 0.9
      duration        = "60s"

      aggregations {
        alignment_period   = "60s"
        per_series_aligner = "ALIGN_MEAN"
      }
    }
  }

  notification_channels = [google_monitoring_notification_channel.email.id]
  enabled               = true
}
