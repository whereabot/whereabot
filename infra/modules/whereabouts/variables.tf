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
  description = "Canonical relay URL: https://<relay_domain>, or the Cloud Run URI (a service cannot read its own URI)."
  type        = string
}

variable "relay_domain" {
  description = "Custom relay host mapped to Cloud Run (needs Google domain verification), or null for none."
  type        = string
  default     = null
}

variable "legacy_relay_urls" {
  description = "Old relay base URLs the relay still answers RFC 9728 metadata for (appended to WB_PUBLIC_URL)."
  type        = list(string)
  default     = []
}

variable "legacy_site_hosts" {
  description = "Old landing hosts whose /oauth/consent/ stays in the Supabase redirect allow list during a move."
  type        = list(string)
  default     = []
}

variable "image" {
  description = "Relay container image, full path with tag (set by scripts/deploy.sh)."
  type        = string
}
