#!/usr/bin/env bash
# shellcheck disable=SC1091,SC2329
# End-to-end check of the local stack and relay from dev-up.sh. Prints PASS/FAIL; exits 1 on failure.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELAY="${RELAY_URL:-http://127.0.0.1:${WB_PORT:-8787}}"
set -a
. "$ROOT/server/.env.local"
set +a
API="$SUPABASE_URL"
ANON="$SUPABASE_ANON_KEY"
SERVICE="$SUPABASE_SERVICE_ROLE_KEY"
FAILED=0
PHONE_PID=""
WS_DIR="$(mktemp -d)"
PHONE_LOG="$(mktemp)"
cleanup() {
  [[ -n "$PHONE_PID" ]] && kill "$PHONE_PID" 2>/dev/null && wait "$PHONE_PID" 2>/dev/null
  rm -rf "$WS_DIR" "$PHONE_LOG"
}
trap cleanup EXIT
ok() { # ok <label> <command...>: prints PASS/FAIL, returns the command's status
  local label=$1
  shift
  if "$@"; then echo "PASS $label"; return 0; fi
  echo "FAIL $label"
  FAILED=1
  return 1
}
set_all() { # every argument is a non-empty value that is not "null"
  local v
  for v; do [[ -n "$v" && "$v" != null ]] || return 1; done
}
make_user() { # prints: id email password
  local email pw
  email="e2e-$(openssl rand -hex 4)@example.com"
  pw="$(openssl rand -hex 16)"
  curl -s -X POST "$API/auth/v1/admin/users" -H "apikey: $SERVICE" \
    -H "Authorization: Bearer $SERVICE" -H 'Content-Type: application/json' \
    -d "{\"email\":\"$email\",\"password\":\"$pw\",\"email_confirm\":true}" |
    jq -r --arg e "$email" --arg p "$pw" '"\(.id // "null") \($e) \($p)"'
}
sign_in() { # <email> <password> -> access token
  curl -s -X POST "$API/auth/v1/token?grant_type=password" -H "apikey: $ANON" \
    -H 'Content-Type: application/json' \
    -d "{\"email\":\"$1\",\"password\":\"$2\"}" | jq -r '.access_token // empty'
}
make_key() { # <access token> -> plaintext agent key
  curl -s -X POST "$API/rest/v1/rpc/create_agent_key" -H "apikey: $ANON" \
    -H "Authorization: Bearer $1" -H 'Content-Type: application/json' \
    -d '{"p_name":"e2e"}' | jq -r 'if type == "string" then . else empty end'
}
mcp_call() { # <agent key> <json-rpc body> -> sets STATUS and BODY
  local out
  out="$(curl -s -w '\n%{http_code}' -X POST "$RELAY/mcp" -H "Authorization: Bearer $1" \
    -H 'Accept: application/json, text/event-stream' -H 'Content-Type: application/json' \
    -d "$2")"
  STATUS="${out##*$'\n'}"
  BODY="${out%$'\n'*}"
}
# (a) two users, so the isolation check has a second owner
read -r A_ID A_EMAIL A_PW < <(make_user)
read -r B_ID B_EMAIL B_PW < <(make_user)
ok "(a) create test users" set_all "$A_ID" "$B_ID" || exit 1
# (b) sign in
A_TOK="$(sign_in "$A_EMAIL" "$A_PW")"
B_TOK="$(sign_in "$B_EMAIL" "$B_PW")"
ok "(b) sign in both users" set_all "$A_TOK" "$B_TOK" || exit 1
# (c) agent keys
A_KEY="$(make_key "$A_TOK")"
B_KEY="$(make_key "$B_TOK")"
ok "(c) create agent keys" set_all "$A_KEY" "$B_KEY" || exit 1
A_KEY_ID="$(curl -s "$API/rest/v1/agent_keys?select=id" -H "apikey: $ANON" -H "Authorization: Bearer $A_TOK" | jq -r '.[0].id // empty')"
# (d) fake phone for user A
ok "(d) install ws for fake phone" npm install --silent --prefix "$WS_DIR" ws
WS_PREFIX="$WS_DIR" WS_URL="${RELAY/http/ws}/v1/device" PHONE_TOKEN="$A_TOK" \
  node "$ROOT/scripts/fake-phone.mjs" >"$PHONE_LOG" 2>&1 &
PHONE_PID=$!
for _ in $(seq 50); do grep -q connected "$PHONE_LOG" && break; sleep 0.2; done
ok "(d) fake phone connects" grep -q connected "$PHONE_LOG"
# (e) MCP initialize, then get_phone_location with user A's key
INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"e2e","version":"0"}}}'
CALL='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_phone_location","arguments":{"timeout_secs":10}}}'
mcp_call "$A_KEY" "$INIT"
ok "(e) mcp initialize" test "$STATUS" = 200
mcp_call "$A_KEY" "$CALL"
ok "(e) get_phone_location returns 32.0853" grep -q '32\.0853' <<<"$BODY"
# (f) negative checks
mcp_call "wb_$(openssl rand -hex 24)" "$CALL"
ok "(f) random key gets 401" test "$STATUS" = 401
mcp_call "$B_KEY" "$CALL"
ok "(f) other user's key gets tool error" grep -q '"isError":true' <<<"$BODY"
ok "(f) other user's key gets no fix" test "$(grep -c '32\.0853' <<<"$BODY")" = 0
REV="$(curl -s -o /dev/null -w '%{http_code}' -X POST "$API/rest/v1/rpc/revoke_agent_key" \
  -H "apikey: $ANON" -H "Authorization: Bearer $A_TOK" -H 'Content-Type: application/json' \
  -d "{\"p_id\":\"$A_KEY_ID\"}")"
ok "(f) revoke RPC accepted" test "${REV:0:1}" = 2
mcp_call "$A_KEY" "$CALL"
ok "(f) revoked key gets 401" test "$STATUS" = 401
exit "$FAILED"
