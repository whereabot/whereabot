output "relay_url" {
  value = module.whereabouts.relay_url
}

output "supabase_url" {
  value = module.whereabouts.supabase_url
}

output "supabase_anon_key" {
  value     = module.whereabouts.supabase_anon_key
  sensitive = true
}

output "supabase_project_ref" {
  value = module.whereabouts.supabase_project_ref
}

output "db_url" {
  value     = module.whereabouts.db_url
  sensitive = true
}

output "artifact_repository" {
  value = module.whereabouts.artifact_repository
}

# Self link of the relay VM; deploy.yml resets it so it boots the new image.
output "relay_vm" {
  value = module.whereabouts.relay_vm
}

output "relay_vm_ip" {
  description = "Static IP of the relay VM: the A record in infra/dns/ at cutover."
  value       = module.whereabouts.relay_vm_ip
}
