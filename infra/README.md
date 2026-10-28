# Whereabouts infrastructure

OpenTofu. Per environment: a Supabase project, a Cloud Run relay, an artifact registry and secrets. Code: [`envs/`](envs) and [`modules/whereabouts/`](modules/whereabouts).

## Live URLs

- Prod relay: https://whereabouts-relay-prod-yz6gmxu7na-ew.a.run.app
- Dev relay: https://whereabouts-relay-dev-yz6gmxu7na-ew.a.run.app
- Landing page: https://whereabouts.ofek-gabay-he.workers.dev

This is the only list of live URLs. Each relay URL is the `relay_url` output of its environment. Update this list after a new deploy changes a URL.

## Prerequisites (break-glass only)

- `tofu`, `gcloud`, `docker` (buildx), `SUPABASE_ACCESS_TOKEN` and `TF_VAR_google_client_secret` in env, ADC login.
- Copy `envs/<env>/terraform.tfvars.example` to `terraform.tfvars` (git-ignored).

## Deploy the relay

A push to `main` that touches `server/**` or `infra/**` runs [`deploy.yml`](../.github/workflows/deploy.yml): test, build one image tagged with the commit SHA, push it to both registries, apply dev, then apply prod after a reviewer approves the `production` environment. Auth to GCP is keyless (Workload Identity Federation). Applies never overlap (concurrency group per env).

- Keep one relay instance. Phone sockets live in memory. See [architecture limits](../docs/architecture.md#limits-today).
- Migrations have their own workflow (below).
- Rollback: revert the commit, or run `deploy` by hand (workflow_dispatch) with `image_sha` set to an older pushed SHA. That skips the build and applies the current `infra/` with the old image.
- Break-glass (CI down): `scripts/deploy.sh <dev|prod>` builds and applies from your laptop.

### One-time setup (human)

1. Create the state bucket `gs://whereabouts-tfstate` and the envs (see Bootstrap below). Both registries must exist before the first CI run.
2. Apply the identity: `tofu -chdir=infra/bootstrap init && tofu -chdir=infra/bootstrap apply`. Code: [`bootstrap/`](bootstrap).
3. Set repo variables from its outputs and your accounts: `WIF_PROVIDER`, `DEPLOYER_SA`, `GCP_PROJECT`, `GCP_REGION`, `SUPABASE_ORGANIZATION_ID`, `GOOGLE_CLIENT_ID`. The workflow lists every name.
4. Set secrets `GOOGLE_CLIENT_SECRET` and `SUPABASE_ACCESS_TOKEN` in both GitHub environments `dev` and `production`.
5. In GitHub settings, add required reviewers to the `production` environment.
6. Add the APNs key: after the first apply creates the secret `apns-key`, run `gcloud secrets versions add apns-key --data-file=<AuthKey.p8>`, then redeploy the relay. Until then push is off. Key id, team and topic are the `apns_*` variables in [`modules/whereabouts/push.tf`](modules/whereabouts/push.tf).

## Bootstrap (once per env)

1. Create the state bucket that `envs/<env>/backend.tf` names: `gcloud storage buckets create gs://<bucket>`.
2. Enable APIs and the registry first. The image must exist before the relay: `tofu -chdir=envs/<env> init`, then `apply -target=google_artifact_registry_repository.whereabouts`.

## Deploy the landing page

The landing page source is `site/`. Deploy it to Cloudflare Workers static assets.

- Copy `site/` into a new empty directory. Deploy that copy with Wrangler, and pass the copy explicitly as the assets path.
- Never deploy from the repo root. It would publish files outside `site/`.

## Migrations

- The `migrations` workflow ([`.github/workflows/migrations.yml`](../.github/workflows/migrations.yml)) applies `supabase/migrations/` on push to `main`: dev first, then production.
- Never run `supabase db push` by hand against dev or prod.
- Each GitHub environment (`dev`, `production`) holds the secrets and variable the workflow reads. The names are in the workflow file.
- How to add a migration: [contributing](../docs/contributing.md#adding-a-migration).
