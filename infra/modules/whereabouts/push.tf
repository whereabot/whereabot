# APNs push (wakes the phone for a location request). The key value is added by hand, never in state:
# see infra/README.md, "Add the APNs key".

variable "apns_key_id" {
  description = "APNs auth key id (Apple Developer > Keys)."
  type        = string
  default     = "T25QURJ5YD"
}

variable "apns_team_id" {
  description = "Apple Developer team id."
  type        = string
  default     = "V72CW46B5V"
}

variable "apns_topic" {
  description = "App bundle id; the APNs topic."
  type        = string
  default     = "io.whereabot.app"
}

variable "apns_production" {
  description = "true = api.push.apple.com (App Store and TestFlight builds)."
  type        = bool
  default     = true
}

locals {
  apns_env = {
    APNS_KEY_ID     = var.apns_key_id
    APNS_TEAM_ID    = var.apns_team_id
    APNS_TOPIC      = var.apns_topic
    APNS_PRODUCTION = tostring(var.apns_production)
  }
}

resource "google_secret_manager_secret" "apns_key" {
  secret_id = "apns-key"
  replication {
    auto {}
  }
  depends_on = [google_project_service.apis]
}

resource "google_secret_manager_secret_iam_member" "relay_apns_key" {
  secret_id = google_secret_manager_secret.apns_key.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.relay.email}"
}
