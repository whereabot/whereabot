#!/usr/bin/env bash
# Stage the Worker's assets (landing, docs, /app) in a fresh temp dir and print its path.
# Needs built docs-site/.vitepress/dist and app/dist-web. Env: SUPABASE_URL, SUPABASE_ANON_KEY;
# optional RELAY_URL (wss://...) replaces the prod relay in the /app CSP (previews use the dev relay).
# Used by .github/workflows/site-pr-preview.yml.
set -euo pipefail
: "${SUPABASE_URL:?set SUPABASE_URL}" "${SUPABASE_ANON_KEY:?set SUPABASE_ANON_KEY}"
cd "$(dirname "$0")/.."
STAGE=$(mktemp -d)
cp -R site "$STAGE/site"
# Landing and consent link design-system/*.css; only the stylesheets ship (no gallery page).
mkdir "$STAGE/site/design-system"
cp design-system/*.css "$STAGE/site/design-system/"
test -f "$STAGE/site/design-system/design-system.css"
sed -i.bak "s|__SUPABASE_URL__|${SUPABASE_URL}|g; s|__SUPABASE_ANON_KEY__|${SUPABASE_ANON_KEY}|g" "$STAGE/site/oauth/consent/supabase-config.js"
rm "$STAGE/site/oauth/consent/supabase-config.js.bak"
if [ -n "${RELAY_URL:-}" ]; then
  sed -i.bak "s|wss://relay.whereabot.io|${RELAY_URL}|g" "$STAGE/site/_headers"
  rm "$STAGE/site/_headers.bak"
  grep -qF "connect-src 'self' https://*.supabase.co ${RELAY_URL};" "$STAGE/site/_headers"
fi
if grep -rEn '__[A-Z0-9_]+__' "$STAGE/site" >&2; then echo "unfilled placeholder" >&2; exit 1; fi
# After the placeholder check: the docs bundle contains names like __VP_HASH_MAP__.
cp -R docs-site/.vitepress/dist "$STAGE/site/docs"
cp -R app/dist-web "$STAGE/site/app"
echo "$STAGE"
