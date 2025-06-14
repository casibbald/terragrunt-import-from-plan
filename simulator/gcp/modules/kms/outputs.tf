// outputs.tf

output "crypto_key_name" {
  value       = google_kms_crypto_key.example.name
  description = "Name of the created crypto key"
}

