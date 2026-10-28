#!/usr/bin/env bash
# shellcheck disable=SC1090
# Starts local Supabase, resets the DB to the migrations, writes server/.env.local,
# prints the app/.env.local lines, then runs the relay in the foreground.
set -euo pipefail
export PATH="$HOME/.local/bin:$PATH"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/Users/ofek/dev/.cargo-target/whereabouts-server}"
ENV_FILE="$ROOT/server/.env.local"
BIN="$CARGO_TARGET_DIR/debug/whereabouts-server"
WB_PORT="${WB_PORT:-8787}"

cd "$ROOT"
supabase start
supabase db reset --local

# Values come from the CLI as KEY="value" lines. The service role key never leaves this file.
status="$(supabase status -o env)"
env_of() { printf '%s\n' "$status" | sed -n "s/^$1=\"\{0,1\}\([^\"]*\)\"\{0,1\}$/\1/p"; }
API_URL="$(env_of API_URL)"
ANON_KEY="$(env_of ANON_KEY)"
SERVICE_ROLE_KEY="$(env_of SERVICE_ROLE_KEY)"
if [[ -z "$API_URL" || -z "$ANON_KEY" || -z "$SERVICE_ROLE_KEY" ]]; then
  echo "error: API_URL, ANON_KEY or SERVICE_ROLE_KEY missing from supabase status" >&2
  exit 1
fi

umask 077
cat >"$ENV_FILE" <<EOF
SUPABASE_URL=$API_URL
SUPABASE_ANON_KEY=$ANON_KEY
SUPABASE_SERVICE_ROLE_KEY=$SERVICE_ROLE_KEY
WHEREABOUTS_ADDR=127.0.0.1:$WB_PORT
EOF
echo "wrote server/.env.local (service role key not shown)"

cat <<EOF

# Copy these lines into app/.env.local (Android emulator reaches the host at 10.0.2.2)
VITE_SUPABASE_URL=http://10.0.2.2:54321
VITE_SUPABASE_ANON_KEY=$ANON_KEY
VITE_RELAY_URL=ws://10.0.2.2:$WB_PORT
VITE_DEV_LOGIN=true

EOF

# Build first, then exec the binary so this shell's PID is the relay's PID.
(cd "$ROOT/server" && cargo build)
set -a
. "$ENV_FILE"
set +a
echo $$ >/tmp/wb-relay.pid
exec "$BIN"
