// outputs.tf

output "topic_name" {
  value       = google_pubsub_topic.example.name
  description = "The name of the created Pub/Sub topic"
}

output "subscription_name" {
  value       = google_pubsub_subscription.example.name
  description = "The name of the created Pub/Sub subscription"
}


