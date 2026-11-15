# Roles match what infra/modules/whereabouts manages: Cloud Run, the relay VM, registry,
# relay service accounts, secrets (+ their IAM), API enablement and alerts.
locals {
  deployer_roles = [
    "roles/run.admin",
    # Relay VM (relay_vm.tf): the VM, its reset and serial output in deploy.yml, the static IP,
    # the firewall rule.
    "roles/compute.instanceAdmin.v1",
    "roles/compute.networkAdmin",
    "roles/compute.securityAdmin",
    "roles/artifactregistry.admin",
    "roles/secretmanager.admin",
    "roles/iam.serviceAccountAdmin",
    "roles/serviceusage.serviceUsageAdmin",
    "roles/monitoring.editor",
    # Log-match alert policies also need logging.notificationRules.create.
    "roles/logging.configWriter",
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

# The relay VM writes its container logs to Cloud Logging. A project grant, so it lives here:
# the deployer cannot change project IAM. Prod only: only prod runs the VM, and a grant to a
# missing service account (dev) would fail the apply.
resource "google_project_iam_member" "relay_vm_logs" {
  project = var.gcp_project
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:whereabouts-relay-prod@${var.gcp_project}.iam.gserviceaccount.com"
}
