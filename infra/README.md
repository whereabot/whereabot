# Whereabouts infrastructure

OpenTofu. Per environment: a Supabase project, a Cloud Run relay, an artifact registry and secrets. Code: [`envs/`](envs) and [`modules/whereabouts/`](modules/whereabouts).

## Live URLs

- Prod relay: https://whereabouts-relay-prod-yz6gmxu7na-ew.a.run.app
- Dev relay: https://whereabouts-relay-dev-yz6gmxu7na-ew.a.run.app
- Landing page: https://whereabouts.ofek-gabay-he.workers.dev

This is the only list of live URLs. Each relay URL is the `relay_url` output of its environment. Update this list after a new deploy changes a URL.

## Prerequisites

- `tofu`, `gcloud`, `docker` (buildx).
- `SUPABASE_ACCESS_TOKEN` in env. Never in tfvars.
- `gcloud auth application-default login` (ADC for Google).
- `TF_VAR_google_client_secret` in env.
- Copy `envs/<env>/terraform.tfvars.example` to `terraform.tfvars`.

## Bootstrap (once per env)

1. Create the state bucket that `envs/<env>/backend.tf` names: `gcloud storage buckets create gs://<bucket>`.
2. Enable APIs and the registry first. The image must exist before the relay: `tofu -chdir=envs/<env> init`, then `apply -target=google_artifact_registry_repository.whereabouts`.

## Deploy the relay

- Run `scripts/deploy.sh dev` or `scripts/deploy.sh prod`. The script builds and pushes the image, then applies it.
- Keep one relay instance. Phone sockets live in memory. See [architecture limits](../docs/architecture.md#limits-today).
- The deploy script does not apply migrations.

## Deploy the landing page

The landing page source is `site/`. Deploy it to Cloudflare Workers static assets.

- Copy `site/` into a new empty directory. Deploy that copy with Wrangler, and pass the copy explicitly as the assets path.
- Never deploy from the repo root. It would publish files outside `site/`.

## Migrations

- The `migrations` workflow ([`.github/workflows/migrations.yml`](../.github/workflows/migrations.yml)) applies `supabase/migrations/` on push to `main`: dev first, then production.
- Never run `supabase db push` by hand against dev or prod.
- Each GitHub environment (`dev`, `production`) holds the secrets and variable the workflow reads. The names are in the workflow file.
- How to add a migration: [contributing](../docs/contributing.md#adding-a-migration).
