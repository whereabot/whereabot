resource "google_cloud_run_v2_service" "relay" {
  name                = "whereabouts-relay-${var.env}"
  location            = var.gcp_region
  ingress             = "INGRESS_TRAFFIC_ALL"
  deletion_protection = var.env == "prod"

  template {
    service_account                  = google_service_account.relay.email
    timeout                          = "3600s"
    max_instance_request_concurrency = 1000
    session_affinity                 = true

    # The relay holds phone WebSockets in memory: exactly one instance.
    scaling {
      min_instance_count = 1
      max_instance_count = 1
    }

    containers {
      image = var.image

      ports {
        container_port = 8080
      }

      resources {
        cpu_idle = false # keep CPU on while sockets are open
      }

      env {
        name  = "SUPABASE_URL"
        value = "https://${local.ref}.supabase.co"
      }
      env {
        name  = "SUPABASE_ISSUER"
        value = "https://${local.ref}.supabase.co/auth/v1"
      }
      env {
        name  = "WB_PUBLIC_URL"
        value = join(",", concat([var.relay_public_url], var.legacy_relay_urls))
      }
      env {
        name  = "RUST_LOG"
        value = "info"
      }
      # Push (see push.tf). Only when apns_enabled: Cloud Run refuses a secret with no version.
      dynamic "env" {
        for_each = var.apns_enabled ? ["APNS_KEY"] : []
        content {
          name = env.value
          value_source {
            secret_key_ref {
              secret  = google_secret_manager_secret.apns_key.secret_id
              version = "latest"
            }
          }
        }
      }
      env {
        name = "SUPABASE_ANON_KEY"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.relay["SUPABASE_ANON_KEY"].secret_id
            version = "latest"
          }
        }
      }
      dynamic "env" {
        for_each = local.apns_env
        content {
          name  = env.key
          value = env.value
        }
      }

      startup_probe {
        http_get {
          path = "/health"
        }
      }
      liveness_probe {
        http_get {
          path = "/health"
        }
      }
    }
  }

  depends_on = [google_project_service.apis, google_secret_manager_secret_iam_member.relay_access]
}

# Public URL. The relay does its own auth.
resource "google_cloud_run_v2_service_iam_member" "public" {
  name     = google_cloud_run_v2_service.relay.name
  location = google_cloud_run_v2_service.relay.location
  role     = "roles/run.invoker"
  member   = "allUsers"
}
