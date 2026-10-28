resource "random_password" "db" {
  length  = 32
  special = false # keeps the password URL-safe for db_url
}

resource "supabase_project" "this" {
  organization_id   = var.supabase_organization_id
  name              = "whereabouts-${var.env}"
  database_password = random_password.db.result
  region            = var.supabase_region

  lifecycle {
    ignore_changes = [database_password]
  }
}

resource "supabase_settings" "this" {
  project_ref = local.ref

  auth = jsonencode({
    site_url                  = "whereabouts://auth-callback"
    uri_allow_list            = "whereabouts://auth-callback"
    disable_signup            = false
    external_google_enabled   = true
    external_google_client_id = var.google_client_id
    external_google_secret    = var.google_client_secret
  })
}

data "supabase_apikeys" "this" {
  project_ref = local.ref
}
