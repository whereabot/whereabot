output "relay_url" {
  description = "Public URL of the relay."
  value       = google_cloud_run_v2_service.relay.uri
}

output "supabase_url" {
  description = "Supabase API URL."
  value       = "https://${supabase_project.this.id}.supabase.co"
}

output "supabase_anon_key" {
  description = "Supabase anon key (public, but marked sensitive by the provider)."
  value       = data.supabase_apikeys.this.anon_key
  sensitive   = true
}

output "supabase_project_ref" {
  description = "Supabase project reference."
  value       = supabase_project.this.id
}

output "db_url" {
  description = "Direct Postgres URL (read by the migrations workflow setup; never push migrations by hand)."
  value       = "postgresql://postgres:${random_password.db.result}@db.${supabase_project.this.id}.supabase.co:5432/postgres"
  sensitive   = true
}

output "artifact_repository" {
  description = "Image prefix for scripts/deploy.sh, without tag."
  value       = "${var.gcp_region}-docker.pkg.dev/${var.gcp_project}/${google_artifact_registry_repository.whereabouts.repository_id}/relay"
}
