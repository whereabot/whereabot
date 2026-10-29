# Whereabouts infrastructure

OpenTofu. Per environment: a Supabase project, a Cloud Run relay, an artifact registry and secrets. Code: [`envs/`](envs) and [`modules/whereabouts/`](modules/whereabouts).

Why the setup looks like this, and the traps we hit: [operations](../docs/operations.md).

## Live URLs

- Prod relay: https://relay.whereabot.io (MCP: https://relay.whereabot.io/mcp)
- Dev relay: https://whereabouts-relay-dev-yz6gmxu7na-ew.a.run.app
- Landing page, docs and OAuth consent: https://whereabot.io

This is the only list of live URLs. Each relay URL is the `relay_url` output of its environment. Update this list after a new deploy changes a URL.

Transition: the old prod hosts (`whereabouts-relay-prod-yz6gmxu7na-ew.a.run.app`, `whereabouts.ofek-gabay-he.workers.dev`) still work. The `legacy_*` inputs in [`envs/prod/main.tf`](envs/prod/main.tf) keep them; remove them once every client and app build uses the new hosts.

## DNS (whereabot.io)

The zone is in the Cloudflare account "whereabot", not the personal one. [`dns/`](dns) holds the relay CNAME and the Google verification TXT. A human applies it with `CLOUDFLARE_API_TOKEN` (Zone DNS: Edit on whereabot.io). The apex record belongs to the Worker custom domain, which `site-cloudflare-deploy` creates.

The relay host is a Cloud Run domain mapping (`relay_domain`, [`cloud_run_domain_mapping.tf`](modules/whereabouts/cloud_run_domain_mapping.tf)). Google must see whereabot.io as verified, with the CI deployer service account as an owner in Search Console.

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
6. Add the APNs key: after the first apply creates the secret `whereabouts-<env>-APNS_KEY`, add the `.p8` as its first version with `gcloud secrets versions add`, then redeploy the relay. Until then push is off. Key id, team and topic are the `apns_*` variables in [`modules/whereabouts/push.tf`](modules/whereabouts/push.tf).

## Bootstrap (once per env)

1. Create the state bucket that `envs/<env>/backend.tf` names: `gcloud storage buckets create gs://<bucket>`.
2. Enable APIs and the registry first. The image must exist before the relay: `tofu -chdir=envs/<env> init`, then `apply -target=google_artifact_registry_repository.whereabouts`.

## Deploy the landing page

The [`site-cloudflare-deploy`](../.github/workflows/site-cloudflare-deploy.yml) workflow deploys `site/` on push to `main`. It deploys a clean copy, so files outside `site/` never publish.

- Setup: repo variable `CLOUDFLARE_ACCOUNT_ID` = the "whereabot" account, which owns the whereabot.io zone (the job stays idle without it); in environment `production`, secret `CLOUDFLARE_API_TOKEN` for that account (Workers Scripts: Edit, Workers Routes: Edit, Zone DNS: Edit on whereabot.io) and variables `SUPABASE_URL`, `SUPABASE_ANON_KEY`.
- The deploy attaches the custom domain whereabot.io to the Worker.
- Break-glass: copy `site/` into an empty directory and deploy that copy with Wrangler, passing it explicitly as the assets path.

## Release the iOS app

The [`ios-testflight-release`](../.github/workflows/ios-testflight-release.yml) workflow builds, signs and uploads to TestFlight when an `app-vX.Y.Z` tag is pushed.

- Release: run [`scripts/release-app-version.sh`](../scripts/release-app-version.sh) `X.Y.Z`, merge to `main`, then push the tag it prints.
- Setup, environment `testflight`: secret `ASC_API_KEY_P8`; variables `ASC_KEY_ID`, `ASC_ISSUER_ID`, `APPLE_TEAM_ID`, `SUPABASE_URL`, `SUPABASE_ANON_KEY`, `RELAY_URL` (prod: `wss://relay.whereabot.io`). The API key needs the App Manager role or higher.
- A DUPLICATE error means TestFlight has that version: bump and tag again.

## Migrations

- The `migrations` workflow ([`.github/workflows/migrations.yml`](../.github/workflows/migrations.yml)) applies `supabase/migrations/` on push to `main`: dev first, then production.
- Never run `supabase db push` by hand against dev or prod.
- Each GitHub environment (`dev`, `production`) holds the secrets and variable the workflow reads. The names are in the workflow file.
- How to add a migration: [contributing](../docs/contributing.md#adding-a-migration).
