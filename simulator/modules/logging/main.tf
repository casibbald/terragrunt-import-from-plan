// main.tf

resource "google_logging_project_sink" "example" {
  project              = var.project_id
  name                  = local.sink_name
  destination           = "storage.googleapis.com/${var.bucket_name}"
  filter                = "resource.type=\"gce_instance\""
  unique_writer_identity = true
}
resource "google_logging_metric" "example" {
    project     = var.project_id
  name        = local.metric_name
  description = "Log-based metric for tracking errors"

  filter = "severity>=ERROR"

  metric_descriptor {
    metric_kind = "DELTA"
    value_type  = "INT64"
  }
}

