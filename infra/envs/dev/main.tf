terraform {
  required_providers {
    supabase = {
      source  = "supabase/supabase"
      version = "~> 1.11"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
  }
}

# Supabase token: SUPABASE_ACCESS_TOKEN env var. Google auth: ADC (gcloud auth application-default login).
provider "supabase" {}

provider "google" {
  project = var.gcp_project
  region  = var.gcp_region
}

module "whereabouts" {
  source = "../../modules/whereabouts"

  env                      = "dev"
  gcp_project              = var.gcp_project
  gcp_region               = var.gcp_region
  supabase_organization_id = var.supabase_organization_id
  supabase_region          = var.supabase_region
  google_client_id         = var.google_client_id
  google_client_secret     = var.google_client_secret
  image                    = var.image
  site_host                = "whereabouts.ofek-gabay-he.workers.dev"
  relay_public_url         = "https://whereabouts-relay-dev-yz6gmxu7na-ew.a.run.app"
}

variable "gcp_project" {
  type = string
}

variable "gcp_region" {
  type    = string
  default = "europe-west1"
}

variable "supabase_organization_id" {
  type = string
}

variable "supabase_region" {
  type    = string
  default = "eu-central-1"
}

variable "google_client_id" {
  type = string
}

variable "google_client_secret" {
  type      = string
  sensitive = true
}

variable "image" {
  description = "Set by scripts/deploy.sh."
  type        = string
}

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
