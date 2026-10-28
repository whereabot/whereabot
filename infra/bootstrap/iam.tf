# Roles match what infra/modules/whereabouts manages: Cloud Run, registry,
# relay service accounts, secrets (+ their IAM) and API enablement.
locals {
  deployer_roles = [
    "roles/run.admin",
    "roles/artifactregistry.admin",
    "roles/secretmanager.admin",
    "roles/iam.serviceAccountAdmin",
    "roles/serviceusage.serviceUsageAdmin",
  ]
}

resource "google_project_iam_member" "deployer" {
  for_each = toset(local.deployer_roles)
  project  = var.gcp_project
  role     = each.key
  member   = "serviceAccount:${google_service_account.deployer.email}"
}

# Cloud Run deploys act as the relay runtime service accounts.
resource "google_project_iam_member" "deployer_act_as" {
  project = var.gcp_project
  role    = "roles/iam.serviceAccountUser"
  member  = "serviceAccount:${google_service_account.deployer.email}"
}

resource "google_storage_bucket_iam_member" "state" {
  bucket = var.state_bucket
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.deployer.email}"
}
