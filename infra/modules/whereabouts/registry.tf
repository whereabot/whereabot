resource "google_artifact_registry_repository" "whereabouts" {
  location = var.gcp_region
  # Envs share one GCP project; dev keeps the first name it was created with.
  repository_id = var.env == "dev" ? "whereabouts" : "whereabouts-${var.env}"
  format        = "DOCKER"
  depends_on    = [google_project_service.apis]
}
