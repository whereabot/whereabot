# Infrastructure

OpenTofu. Per environment: a Supabase project, a Cloud Run relay, an artifact registry and secrets. Code: [`envs/`](envs) and [`modules/whereabouts/`](modules/whereabouts).

Why the setup looks like this, and the traps we hit: [operations](../docs/operations.md).

This page describes the whereabot.io deployment. To self-host, replace the project ids, bucket names, domain and accounts in [`envs/`](envs), [`dns/`](dns) and the repo variables with your own.

## Live URLs

- Prod relay: https://relay.whereabot.io (MCP: https://relay.whereabot.io/mcp)
- Dev relay: https://whereabouts-relay-dev-yz6gmxu7na-ew.a.run.app
- Landing page, docs and OAuth consent: https://whereabot.io
- Dev landing page, docs and OAuth consent: https://dev.whereabot.io

This is the only list of live URLs. Each relay URL is the `relay_url` output of its environment. Update this list after a new deploy changes a URL.

Transition: the old prod hosts still work. The `legacy_*` inputs in [`envs/prod/main.tf`](envs/prod/main.tf) keep them; remove them once every client and app build uses the new hosts.

## DNS (whereabot.io)

The zone has its own Cloudflare account. [`dns/`](dns) holds the relay CNAME and the Google verification TXT. A human applies it with an API token that can edit that zone's DNS. The apex record belongs to the Worker custom domain, which `site-cloudflare-deploy` creates.

The relay host is a domain mapping (`relay_domain`). Why a human creates it, and how to verify DNS: [operations, domains](../docs/operations.md#domains-dns-and-certificates).

## Prerequisites (break-glass only)

- `tofu`, `gcloud`, `docker` (buildx), `SUPABASE_ACCESS_TOKEN` and `TF_VAR_google_client_secret` in env, ADC login.
- Copy `envs/<env>/terraform.tfvars.example` to `terraform.tfvars` (git-ignored).

## Deploy the relay

A push that touches the relay or infra (paths in the workflow) runs [`deploy.yml`](../.github/workflows/deploy.yml) for the branch's environment ([branch map](../docs/operations.md#delivery-is-gitops)): test, build one image tagged with the commit SHA, push it to that environment's registry, then apply. Prod applies wait for a reviewer to approve the `production` environment. Auth to GCP is keyless (Workload Identity Federation). Applies never overlap (concurrency group per env).

- Keep one relay instance. Phone sockets live in memory. See [architecture limits](../docs/architecture.md#limits-today).
- Migrations have their own workflow (below).
- Rollback: revert the commit, or run `deploy` by hand (workflow_dispatch) on the env's branch with `image_sha` set to an older SHA pushed to that env's registry. That skips the build and applies the current `infra/` with the old image.
- Break-glass (CI down): `scripts/deploy.sh <dev|prod>` builds and applies from your laptop.

### One-time setup (human)

1. Create the state bucket `gs://whereabouts-tfstate` and the envs (see Bootstrap below). Both registries must exist before the first CI run.
2. Apply the identity: `tofu -chdir=infra/bootstrap init && tofu -chdir=infra/bootstrap apply`. Code: [`bootstrap/`](bootstrap).
3. Set the repo variables that [`deploy.yml`](../.github/workflows/deploy.yml) reads, from the bootstrap outputs and your accounts.
4. Set the secrets that `deploy.yml` reads in both GitHub environments `dev` and `production`.
5. In GitHub settings, add required reviewers to the `production` environment. Limit each environment's deployment branches to its branch ([branch map](../docs/operations.md#delivery-is-gitops)).
6. Add the APNs key: after the first apply creates the secret `whereabouts-<env>-APNS_KEY`, add the `.p8` as its first version with `gcloud secrets versions add`, then redeploy the relay. Until then push is off. Key id, team and topic are the `apns_*` variables in [`modules/whereabouts/push.tf`](modules/whereabouts/push.tf).
7. Add the VAPID key (Web Push for the browser app): add the base64url private key as the first version of `whereabouts-<env>-VAPID_PRIVATE_KEY`, set the matching public key as the `VITE_VAPID_PUBLIC_KEY` variable of the GitHub environment, then set `web_push_enabled = true` in `envs/<env>/main.tf`. Code: [`modules/whereabouts/web_push.tf`](modules/whereabouts/web_push.tf).
8. Sign in with Apple: in developer.apple.com, give the Services ID (`apple_services_id`) the Sign in with Apple capability. Add each env's Supabase callback, `https://<project-ref>.supabase.co/auth/v1/callback`, as a return URL and `<project-ref>.supabase.co` as a domain. Put the client secret JWT (from the `.p8` key, expires after 6 months at most) in the `APPLE_CLIENT_SECRET` secret of both environments. While it is empty, Apple sign-in stays off. Code: [`modules/whereabouts/apple-sign-in-variables.tf`](modules/whereabouts/apple-sign-in-variables.tf).

## Bootstrap (once per env)

1. Create the state bucket that `envs/<env>/backend.tf` names: `gcloud storage buckets create gs://<bucket>`.
2. Enable APIs and the registry first. The image must exist before the relay: `tofu -chdir=envs/<env> init`, then `apply -target=google_artifact_registry_repository.whereabouts`.

## Deploy the landing page

The [`site-cloudflare-deploy`](../.github/workflows/site-cloudflare-deploy.yml) workflow deploys `site/` to the branch's Worker: `whereabouts` at whereabot.io from `main`, `whereabouts-dev` at dev.whereabot.io from `dev`. It deploys a clean copy, so files outside `site/` never publish.

- Setup: in each environment (`dev`, `production`), the variables and secrets the workflow reads, for the Cloudflare account that owns the zone. The job stays idle until the account variable is set. The API token needs edit rights on Workers scripts, Workers routes and the zone DNS.
- The deploy attaches the custom domain to the Worker.
- Break-glass: copy `site/` into an empty directory and deploy that copy with Wrangler, passing it explicitly as the assets path.

### PR previews

The [`site-pr-preview`](../.github/workflows/site-pr-preview.yml) workflow deploys each same-repo PR that touches the site, docs or app to a throwaway Worker `whereabouts-pr-<N>` on workers.dev. The preview uses the dev Supabase and the dev relay. The workflow comments the URL on the PR and deletes the Worker when the PR closes.

- Setup (owner, once): register the account's workers.dev subdomain in the Cloudflare dashboard (Workers, onboarding). Create GitHub environment `preview` with secret `CLOUDFLARE_API_TOKEN`, scoped to Workers Scripts: Edit on that account only. Then set repo variable `SITE_PREVIEW` = `true`. Until then the workflow stays idle.
- In environment `preview`, also set variables `SUPABASE_URL` and `SUPABASE_ANON_KEY` to the dev Supabase project's values. The build reads them; environment `dev` accepts only the `dev` branch.

## Release the iOS app

The [`ios-testflight-release`](../.github/workflows/ios-testflight-release.yml) workflow builds, signs and uploads to TestFlight when an `app-vX.Y.Z` tag is pushed.

- Release: run [`scripts/release-app-version.sh`](../scripts/release-app-version.sh) `X.Y.Z`, merge to `dev`, promote to `main`, then push the tag it prints. The release workflows refuse a tag that is not on `main`.
- Setup: in environment `testflight`, the secret and variables the workflow reads. `RELAY_URL` is the prod relay from [Live URLs](#live-urls), with `wss://`. The App Store Connect API key needs the App Manager role or higher.
- A DUPLICATE error means TestFlight has that version: bump and tag again.
- Default is TestFlight only. To also submit to App Store review with automatic release, set repo variable `APP_STORE_SUBMIT=true`, or pass input `submit_for_review: true`. The workflow then waits for Apple processing and submits. Another workflow can call it (`workflow_call`, input `tag`, `secrets: inherit`).
- Without `ASC_KEY_ID` in environment `testflight`, the workflow skips green.
- What's New comes from an annotated tag message (`git tag -a app-vX.Y.Z -m "..."`). A lightweight tag sends none; Apple refuses What's New on the first release.
- Human, once in App Store Connect before the first submission: app privacy answers, age rating, content rights, pricing and availability, App Review contact and demo account, the en-US store text and screenshots. Export compliance is answered by `ITSAppUsesNonExemptEncryption` in [`Info.ios.plist`](../app/src-tauri/Info.ios.plist).

## Migrations

- The `migrations` workflow ([`.github/workflows/migrations.yml`](../.github/workflows/migrations.yml)) applies `supabase/migrations/` to the branch's environment ([branch map](../docs/operations.md#delivery-is-gitops)).
- Never run `supabase db push` by hand against dev or prod.
- Each GitHub environment (`dev`, `production`) holds the secrets and variable the workflow reads. The names are in the workflow file.
- How to add a migration: [contributing](../docs/contributing.md#adding-a-migration).
