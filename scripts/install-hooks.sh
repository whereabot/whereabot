#!/bin/sh
# Installs a pre-push shim that runs the versioned .githooks/pre-push.
set -eu

hooks="$(git rev-parse --git-common-dir)/hooks"
mkdir -p "$hooks"
cat > "$hooks/pre-push" <<'EOF'
#!/bin/sh
hook="$(git rev-parse --show-toplevel)/.githooks/pre-push"
# Branches cut before the hook existed have nothing to run.
[ -x "$hook" ] || exit 0
exec "$hook" "$@"
EOF
chmod +x "$hooks/pre-push"
echo "installed $hooks/pre-push -> .githooks/pre-push"
