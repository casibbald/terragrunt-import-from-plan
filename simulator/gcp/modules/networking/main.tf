// main.tf

resource "google_compute_network" "default" {
    project                 = var.project_id
  name                    = local.vpc_name
  auto_create_subnetworks = false
}

resource "google_compute_subnetwork" "default" {
    project      = var.project_id
  name          = local.subnet_name
  ip_cidr_range = "10.0.0.0/16"
  region        = var.region
  network       = google_compute_network.default.id
}

resource "google_compute_firewall" "allow_ssh" {
    project = var.project_id
  name    = "allow-ssh"
  network = google_compute_network.default.name

  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  source_ranges = ["0.0.0.0/0"]
}

resource "google_compute_address" "static_ip" {
    project = var.project_id
  name   = "static-ip"
  region = var.region
}

resource "google_compute_router" "router" {
  project = var.project_id
  name    = "router"
  region  = var.region
  network = google_compute_network.default.id
}

resource "google_compute_vpn_gateway" "vpn_gw" {
  project = var.project_id
  name    = "vpn-gw"
  region  = var.region
  network = google_compute_network.default.id
}

resource "google_compute_vpn_tunnel" "vpn_tunnel" {
  project           = var.project_id
  name               = "vpn-tunnel"
  region             = var.region
  target_vpn_gateway = google_compute_vpn_gateway.vpn_gw.id
  peer_ip            = "8.8.8.8"
  shared_secret      = "supersecret"
  router             = google_compute_router.router.id
}

