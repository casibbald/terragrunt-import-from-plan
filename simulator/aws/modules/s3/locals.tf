locals {
  bucket_name = "example-storage-bucket-${random_id.bucket_suffix.hex}"
}

resource "random_id" "bucket_suffix" {
  byte_length = 4
} 