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

      # Same env as the VM: relay_env.tf.
      dynamic "env" {
        for_each = local.relay_env
        content {
          name  = env.key
          value = env.value
        }
      }
      dynamic "env" {
        for_each = local.relay_secret_env
        content {
          name = env.key
          value_source {
            secret_key_ref {
              secret  = env.value
              version = "latest"
            }
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
