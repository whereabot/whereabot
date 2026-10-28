# Custom relay host (e.g. relay.whereabot.io) on the Cloud Run front end. Free; WebSockets pass.
# Needs: the domain verified in Google Search Console and the CNAME in infra/dns/ (DNS only).
# A domain owner creates it once (gcloud beta run domain-mappings create) and imports it.
resource "google_cloud_run_domain_mapping" "relay" {
  count    = var.relay_domain == null ? 0 : 1
  name     = var.relay_domain
  location = var.gcp_region

  metadata {
    namespace = var.gcp_project
  }

  spec {
    route_name = google_cloud_run_v2_service.relay.name
  }

  # Created once by a human owner of the domain (the CI account is not a Search Console
  # owner); certificate_mode reads back empty after import and must not force a replacement.
  lifecycle {
    ignore_changes = [spec[0].certificate_mode]
  }
}
