locals {
  project_id = "your-gcp-project-id"
  region     = "europe-west1"
}

terraform {
  source = "${get_repo_root()}/simulator/gcp"
}

inputs = {
  project_id = local.project_id
  region     = local.region
}
