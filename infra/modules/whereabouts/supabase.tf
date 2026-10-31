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

  # OAuth server keys: PATCH /v1/projects/{ref}/config/auth, UpdateAuthConfigBody
  # (https://api.supabase.com/api/v1-json, checked 2026-10-28).
  auth = jsonencode(merge({
    site_url                                = "https://${var.site_host}"
    uri_allow_list                          = join(",", concat(["whereabouts://auth-callback", "https://${var.site_host}/app/"], [for h in concat([var.site_host], var.legacy_site_hosts) : "https://${h}/oauth/consent/"]))
    disable_signup                          = false
    external_google_enabled                 = true
    external_google_client_id               = var.google_client_id
    external_google_secret                  = var.google_client_secret
    oauth_server_enabled                    = true
    oauth_server_authorization_path         = "/oauth/consent/"
    oauth_server_allow_dynamic_registration = true
    # Shares match the grantee by token email: an unconfirmed email must never get a session.
    mailer_autoconfirm = false
    # Sign in with Apple: on only when deploy.yml has the secret, so an empty secret cannot break the apply.
    external_apple_enabled = var.apple_client_secret != ""
    }, var.apple_client_secret == "" ? {} : {
    # Services ID for the web flow; the bundle id lets a native iOS id token in too.
    external_apple_client_id = "${var.apple_services_id},io.whereabot.app"
    external_apple_secret    = var.apple_client_secret
  }))
}

data "supabase_apikeys" "this" {
  project_ref = local.ref
}
