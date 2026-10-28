locals {
  ref = supabase_project.this.id
  # Same key names the relay reads from the environment.
  relay_secret_values = {
    SUPABASE_ANON_KEY         = data.supabase_apikeys.this.anon_key
    SUPABASE_SERVICE_ROLE_KEY = data.supabase_apikeys.this.service_role_key
  }
}
