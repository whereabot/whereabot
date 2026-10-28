output "workload_identity_provider" {
  description = "GitHub repo variable WIF_PROVIDER."
  value       = google_iam_workload_identity_pool_provider.github.name
}

output "deployer_service_account" {
  description = "GitHub repo variable DEPLOYER_SA."
  value       = google_service_account.deployer.email
}
