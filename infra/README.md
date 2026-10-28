# Whereabouts infrastructure

OpenTofu. Per env: Supabase project, Cloud Run relay, Artifact Registry, secrets.

## Prerequisites
- `tofu`, `gcloud`, `docker` (buildx).
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

The script builds and pushes the image, then applies.
Relay runs `min=max=1` instance: it holds WebSockets in memory.

## Migrations
- Migrations apply via the `migrations` workflow on push to `main` (dev, then production).
- Never run `supabase db push` by hand.
- Each GitHub environment (`dev`, `production`) needs secret `SUPABASE_ACCESS_TOKEN`, secret `SUPABASE_DB_PASSWORD` and variable `SUPABASE_PROJECT_REF`.
