# Custom relay host (e.g. relay.whereabot.io) on the Cloud Run front end. Free; WebSockets pass.
# Needs: the domain verified in Google Search Console with the applying identity (the CI
# deployer service account) as an owner, and the CNAME in infra/dns/ (DNS only, not proxied).
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
}
