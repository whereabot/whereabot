variable "env" {
  description = "Environment name: dev or prod."
  type        = string
  validation {
    condition     = contains(["dev", "prod"], var.env)
    error_message = "env must be dev or prod."
  }
}

variable "gcp_project" {
  description = "GCP project ID."
  type        = string
}

variable "gcp_region" {
  description = "GCP region for Cloud Run and Artifact Registry."
  type        = string
  default     = "europe-west1"
}

variable "supabase_organization_id" {
  description = "Supabase organization slug."
  type        = string
}

variable "supabase_region" {
  description = "Supabase project region."
  type        = string
  default     = "eu-central-1"
}

variable "google_client_id" {
  description = "Google OAuth client ID used by Supabase Auth."
  type        = string
}

variable "google_client_secret" {
  description = "Google OAuth client secret used by Supabase Auth."
  type        = string
  sensitive   = true
}

variable "site_host" {
  description = "Landing site host (no scheme). Serves the OAuth consent page at /oauth/consent/."
  type        = string
}

variable "relay_public_url" {
  description = "Relay public URL, https://<service-uri>. Set per env; a Cloud Run service cannot read its own URI."
  type        = string
}

variable "image" {
  description = "Relay container image, full path with tag (set by scripts/deploy.sh)."
  type        = string
}
