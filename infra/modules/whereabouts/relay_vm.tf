# One e2-micro runs the relay container behind Caddy (automatic Let's Encrypt). Free tier: one
# e2-micro, 30 GB standard disk, 1 GB egress a month. Runbook: infra/relay-vm.md.
locals {
  relay_vm        = var.relay_vm_enabled ? 1 : 0
  relay_vm_region = regex("^(.*)-[a-z]$", var.relay_vm_zone)[0]
  relay_vm_tag    = "whereabouts-relay-${var.env}"
}

resource "google_compute_address" "relay" {
  count      = local.relay_vm
  name       = "whereabouts-relay-${var.env}"
  region     = local.relay_vm_region
  depends_on = [google_project_service.apis]
}

resource "google_compute_firewall" "relay_web" {
  count         = local.relay_vm
  name          = "whereabouts-relay-${var.env}-web"
  network       = "default"
  source_ranges = ["0.0.0.0/0"]
  target_tags   = [local.relay_vm_tag]
  # 80 for the ACME HTTP challenge and the HTTPS redirect, 443 for the relay.
  allow {
    protocol = "tcp"
    ports    = ["80", "443"]
  }
  depends_on = [google_project_service.apis]
}

resource "google_compute_instance" "relay" {
  count                     = local.relay_vm
  name                      = "whereabouts-relay-${var.env}"
  machine_type              = "e2-micro"
  zone                      = var.relay_vm_zone
  tags                      = [local.relay_vm_tag]
  allow_stopping_for_update = true

  boot_disk {
    initialize_params {
      image = "cos-cloud/cos-stable" # Container-Optimized OS; updates itself
      size  = 30
      type  = "pd-standard"
    }
  }

  network_interface {
    network = "default"
    access_config {
      nat_ip = google_compute_address.relay[0].address
    }
  }

  # Same identity as Cloud Run: it already reads the relay secrets (secrets.tf, push.tf, web_push.tf).
  service_account {
    email  = google_service_account.relay.email
    scopes = ["cloud-platform"]
  }

  # The script runs on every boot. A deploy changes the image here, then deploy.yml resets the VM.
  metadata = {
    google-logging-enabled = "true"
    startup-script = templatefile("${path.module}/relay_vm_startup.sh.tftpl", {
      project    = var.gcp_project
      image      = var.image
      domain     = var.relay_domain
      env        = local.relay_env
      secret_env = local.relay_secret_env
    })
  }

  lifecycle {
    # A newer COS image must not replace the VM (and drop Caddy's certificates).
    ignore_changes = [boot_disk[0].initialize_params[0].image]
    precondition {
      condition     = var.relay_domain != null
      error_message = "relay_vm_enabled needs relay_domain: Caddy gets the certificate for it."
    }
  }
}

# Pull the relay image. Secret access: the existing grants on this service account.
resource "google_artifact_registry_repository_iam_member" "relay_vm_reader" {
  count      = local.relay_vm
  location   = google_artifact_registry_repository.whereabouts.location
  repository = google_artifact_registry_repository.whereabouts.name
  role       = "roles/artifactregistry.reader"
  member     = "serviceAccount:${google_service_account.relay.email}"
}
