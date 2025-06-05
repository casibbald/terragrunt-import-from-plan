// outputs.tf

output "vpc_name" {
  value       = google_compute_network.default.name
  description = "VPC name"
}

output "subnet_name" {
  value       = google_compute_subnetwork.default.name
  description = "Subnetwork name"
}

output "static_ip" {
  value       = google_compute_address.static_ip.address
  description = "Reserved static IP address"
}

