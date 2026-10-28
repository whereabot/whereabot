resource "google_service_account" "relay" {
  account_id   = "whereabouts-relay-${var.env}"
  display_name = "Whereabouts relay (${var.env})"
}

resource "google_secret_manager_secret" "relay" {
  for_each  = toset(keys(local.relay_secret_values))
  secret_id = "whereabouts-${var.env}-${each.key}"
  replication {
    auto {}
  }
  depends_on = [google_project_service.apis]
}

resource "google_secret_manager_secret_version" "relay" {
  for_each    = toset(keys(local.relay_secret_values))
  secret      = google_secret_manager_secret.relay[each.key].id
  secret_data = local.relay_secret_values[each.key]
}

resource "google_secret_manager_secret_iam_member" "relay_access" {
  for_each  = toset(keys(local.relay_secret_values))
  secret_id = google_secret_manager_secret.relay[each.key].id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.relay.email}"
}
