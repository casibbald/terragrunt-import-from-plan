// main.tf

resource "google_pubsub_topic" "example" {
  name    = local.topic_name
  project = var.project_id
}

resource "google_pubsub_subscription" "example" {
  name  = local.sub_name
  topic = google_pubsub_topic.example.id
  ack_deadline_seconds = 20
  project = var.project_id
}

