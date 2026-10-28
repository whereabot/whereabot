resource "google_artifact_registry_repository" "whereabouts" {
  location      = var.gcp_region
  repository_id = "whereabouts"
  format        = "DOCKER"
  depends_on    = [google_project_service.apis]
}
