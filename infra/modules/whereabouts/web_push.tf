# Web Push (notifies browser/PWA installs of a location request). The VAPID private key is added by
# hand, never in state: see infra/README.md, "Add the VAPID key". The public key is the site's
# VITE_VAPID_PUBLIC_KEY GitHub variable.

variable "web_push_enabled" {
  description = "Wire the VAPID private key into the relay. Turn on only after the secret has a version."
  type        = bool
  default     = false
}

locals {
  web_push_secret_env = var.web_push_enabled ? { VAPID_PRIVATE_KEY = google_secret_manager_secret.vapid_private_key.secret_id } : {}
}

resource "google_secret_manager_secret" "vapid_private_key" {
  secret_id = "whereabouts-${var.env}-VAPID_PRIVATE_KEY"
  replication {
    auto {}
  }
  depends_on = [google_project_service.apis]
}

resource "google_secret_manager_secret_iam_member" "relay_vapid_private_key" {
  secret_id = google_secret_manager_secret.vapid_private_key.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.relay.email}"
}
