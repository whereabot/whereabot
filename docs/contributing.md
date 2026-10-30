# Contributing

## Setup

- Run `scripts/install-hooks.sh` once. The pre-push hook runs `linecheck .`. Install it with `cargo install linecheck`.
- Local stack and end-to-end check: [quick start](../README.md#quick-start-local).
- Regenerate UI screenshots in `docs/screens/` with `node scripts/screens.mjs` (needs Playwright and Chromium; see [screens index](screens/README.md)).

## Rules

- Every file stays at the line limit in [`linecheck.yml`](../linecheck.yml). Split a file by its job when it grows.
- Facts live in code. Docs link to code for routes, limits, ports and config. See [protocol](protocol.md).
- Diagrams are Mermaid blocks in the doc that explains them.

## Adding a migration

1. Add a new SQL file under `supabase/migrations/`. Never edit a migration that already reached `main`.
2. Add or update the pgTAP test under `supabase/tests/`.
3. Test locally. Run `scripts/dev-up.sh` in one terminal. It resets the local database to the migrations. In a second terminal, run `supabase test db`, then `scripts/e2e-local.sh`.
4. Open a PR and merge it to `main`.
5. The `migrations` workflow applies the change to dev first, then production. Check that run. Never run `supabase db push` by hand.

Workflow details: [infra/README.md](../infra/README.md#migrations).

## Pull requests

- Open an issue first for a large change, so we agree on the approach before you build it.
- One change per PR. CI must be green: [`ci.yml`](../.github/workflows/ci.yml) lists every check.
- A change that stores or logs new data updates [privacy](privacy.md).
- Report security issues privately: [security policy](../SECURITY.md). Behavior rules: [code of conduct](../CODE_OF_CONDUCT.md).
- By contributing, you agree to license your work under the [MIT license](../LICENSE).
