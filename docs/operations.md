# Operations

How Whereabot runs in production, and the traps we already hit. The step-by-step runbook is [`infra/README.md`](../infra/README.md). The code and workflows are the source of truth for every value. This page names the files and explains why they are shaped the way they are.

## Delivery is GitOps

Every change reaches an environment through a workflow on its branch. Nobody deploys from a laptop except as break-glass.

| Branch | GitHub environment | Gets changes by |
|--------|--------------------|-----------------|
| `dev` (default branch, PR target) | `dev` | merging a PR |
| `main` | `production` (required reviewers) | merging the promotion PR `dev` → `main` |

This table is the only home of the branch-to-environment map. Each environment's deployment branch policy allows only its branch, so a run from any other ref stops before it deploys. PR previews use environment `preview`. Production only gets commits that already ran in dev. [`promote-dev.yml`](../.github/workflows/promote-dev.yml) keeps one promotion PR open after every push to `dev`. Merge it with a merge commit, never squash: a squash commit on `main` is missing from `dev`, and the branches drift apart. The app release bump commit lands on `main`; [`app-auto-release.yml`](../.github/workflows/app-auto-release.yml) then merges `main` back into `dev`.

| What | Workflow | Trigger |
|------|----------|---------|
| Tests, lint, line cap | [`ci.yml`](../.github/workflows/ci.yml) | every PR and push |
| Database migrations | [`migrations.yml`](../.github/workflows/migrations.yml) | push to `dev` or `main`: CI, then that branch's environment |
| Relay + Supabase config | [`deploy.yml`](../.github/workflows/deploy.yml) | push to `dev` or `main`: CI, image, then that branch's environment |
| Landing page, docs and browser app | [`site-cloudflare-deploy.yml`](../.github/workflows/site-cloudflare-deploy.yml) | push to `dev` or `main`: CI, then that branch's Worker |
| iOS app to TestFlight, then App Store review when variable `APP_STORE_SUBMIT` is `true` | [`ios-testflight-release.yml`](../.github/workflows/ios-testflight-release.yml) | tag `app-vX.Y.Z` on a commit in `main` |
| Android app to Google Play internal testing | [`android-play-release.yml`](../.github/workflows/android-play-release.yml) | tag `app-vX.Y.Z` on a commit in `main`; called by other workflows only when variable `ANDROID_RELEASE` is `true` |

Migrations and deploys are separate workflows with no ordering between them. Keep that in mind for the next section.

## Config that needs a migration ships in two pushes

The Supabase auth config (for example an auth hook) is applied by `deploy.yml`. The Postgres function it points at is created by `migrations.yml`. If both land in one push, the config can reach an environment before the function exists, and the apply fails.

1. First push: the migration only. Promote it to `main` on its own, too: one promotion that carries both is one push to production.
2. Check that `migrations.yml` applied it in dev and production.
3. Second push: the config in [`supabase.tf`](../infra/modules/whereabouts/supabase.tf) that uses it.

New migration files must sort after every migration already applied. CI refuses a file with an older timestamp. See [adding a migration](contributing.md#adding-a-migration).

## Optional secrets stay behind a flag

Cloud Run refuses a revision that mounts a secret with no version. The APNs key secret is created empty, and a human adds the key later. So the relay mounts it only when `apns_enabled` is true. See [`push.tf`](../infra/modules/whereabouts/push.tf) and [`cloud_run.tf`](../infra/modules/whereabouts/cloud_run.tf). Turn the flag on per environment only after the secret has a version.

## Domains, DNS and certificates

The live hosts are listed once, in [infra/README.md, Live URLs](../infra/README.md#live-urls).

- **Relay host.** A Cloud Run domain mapping ([`cloud_run_domain_mapping.tf`](../infra/modules/whereabouts/cloud_run_domain_mapping.tf)). Google lets only a verified owner of the domain create a mapping, and the CI service account is not one. A human domain owner creates the mapping once, then imports it into the prod state. The resource ignores `certificate_mode`, because it reads back empty after import and would force a replacement.
- **DNS records.** In [`infra/dns/`](../infra/dns), applied by a human. The relay CNAME must be DNS only (not proxied). Google issues the certificate and must see the CNAME.
- **Verify, do not trust the dashboard.** A record saved in the Cloudflare dashboard did not persist once. After any DNS change, query a public resolver for the record, then wait until the relay answers over HTTPS. The certificate can take a while after DNS is correct.
- **Site host.** The site workflow attaches the custom domain to the Worker and owns the apex record.

## Release the iOS app

The normal path is a tag. The steps are in [infra/README.md, Release the iOS app](../infra/README.md#release-the-ios-app).

- [`scripts/release-app-version.sh`](../scripts/release-app-version.sh) bumps every version file. Merge that, then push the tag it prints.
- Fallback when CI cannot run: build and upload from a Mac with the same two steps as the workflow (a Tauri iOS build with the App Store Connect export method, then an upload with `altool`). Use the same API key and environment values the workflow reads.

## Release the Android app

The same tag also builds the Android AAB. The workflow uploads it to Play internal testing only when the keystore and Play secrets exist in environment `google-play`. Without them it keeps an unsigned AAB as a run artifact.

- A calling workflow (for example the automatic release) skips the Android job until a human sets the repository variable `ANDROID_RELEASE` to `true`. Set it only after the secrets exist. This stops an unusable unsigned build on every release.
- Play takes only draft releases until the first publish. Set variable `PLAY_RELEASE_STATUS` to `draft` until then.
- Play rejects a version code it already has. Bump and tag again.

## Steps that need a human

Some steps change shared accounts. An agent prepares them, but a human does them or explicitly approves them:

- DNS changes in the Cloudflare account that owns the zone.
- Writing secret values (GitHub environment secrets, Secret Manager versions).
- IAM grants and domain ownership in Google Search Console.
- Approving the `production` environment gate.

After a human step, verify the result (resolver query, `/health`, workflow run) before you continue.
