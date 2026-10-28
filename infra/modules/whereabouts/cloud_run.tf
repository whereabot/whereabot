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
        name  = "RUST_LOG"
        value = "info"
      }
      env {
        name = "SUPABASE_ANON_KEY"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.relay["SUPABASE_ANON_KEY"].secret_id
            version = google_secret_manager_secret_version.relay["SUPABASE_ANON_KEY"].version
          }
        }
      }
      env {
        name = "SUPABASE_SERVICE_ROLE_KEY"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.relay["SUPABASE_SERVICE_ROLE_KEY"].secret_id
            version = google_secret_manager_secret_version.relay["SUPABASE_SERVICE_ROLE_KEY"].version
          }
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
