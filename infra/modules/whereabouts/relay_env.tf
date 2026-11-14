# Relay environment, one home for both hosts: Cloud Run (cloud_run.tf) and the VM (relay_vm.tf).
locals {
  relay_env = merge({
    SUPABASE_URL    = "https://${local.ref}.supabase.co"
    SUPABASE_ISSUER = "https://${local.ref}.supabase.co/auth/v1"
    WB_PUBLIC_URL   = join(",", concat([var.relay_public_url], var.legacy_relay_urls))
    RUST_LOG        = "info"
  }, local.apns_env)

  # Env name -> Secret Manager secret id. Push keys only when enabled (push.tf, web_push.tf):
  # Cloud Run refuses a secret with no version, and the VM boot script stops on one.
  relay_secret_env = merge(
    { SUPABASE_ANON_KEY = google_secret_manager_secret.relay["SUPABASE_ANON_KEY"].secret_id },
    local.apns_secret_env,
    local.web_push_secret_env,
  )
}
