# Whereabouts infrastructure

OpenTofu. Per env: Supabase project, Cloud Run relay, Artifact Registry, secrets.

## Prerequisites
- `tofu`, `gcloud`, `docker` (buildx), `supabase` CLI.
- `SUPABASE_ACCESS_TOKEN` in env. Never in tfvars.
- `gcloud auth application-default login` (ADC for Google).
- `TF_VAR_google_client_secret` in env.
- Copy `envs/<env>/terraform.tfvars.example` to `terraform.tfvars`.

## Bootstrap (once per env)
1. Create state bucket: `gcloud storage buckets create gs://whereabouts-tfstate --location=europe-west1`
2. Enable APIs and repo first (image must exist before relay):
   `tofu -chdir=envs/<env> init && tofu -chdir=envs/<env> apply -target=google_artifact_registry_repository.whereabouts`

## Apply per env
- Dev: `scripts/deploy.sh dev`
- Prod: `scripts/deploy.sh prod`

The script builds and pushes the image, applies, then runs `supabase db push`.
Relay runs `min=max=1` instance: it holds WebSockets in memory.
