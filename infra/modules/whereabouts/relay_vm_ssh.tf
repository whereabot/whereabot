# default-allow-ssh (priority 65534) opens tcp:22 to the internet. This deny beats it at 900.
resource "google_compute_firewall" "relay_ssh_deny" {
  count         = local.relay_vm
  name          = "whereabouts-relay-${var.env}-deny-ssh"
  network       = "default"
  priority      = 900
  direction     = "INGRESS"
  source_ranges = ["0.0.0.0/0"]
  target_tags   = [local.relay_vm_tag]
  deny {
    protocol = "tcp"
    ports    = ["22"]
  }
  depends_on = [google_project_service.apis]
}
