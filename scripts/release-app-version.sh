#!/usr/bin/env bash
# Bump the app version in the three manifests; print the tag command that releases it to TestFlight.
# Usage: scripts/release-app-version.sh 0.1.13   (no commit, no push)
set -euo pipefail
v="${1:?usage: $0 X.Y.Z}"
[[ "$v" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "version must be X.Y.Z, got '$v'" >&2; exit 1; }
cd "$(dirname "$0")/../app"
# -0 slurps the file; s/// without /g changes only the first match: the app's own version.
for f in package.json src-tauri/tauri.conf.json; do
  perl -0pi -e 's/"version": *"[^"]*"/"version": "'"$v"'"/' "$f"
done
perl -0pi -e 's/^version = "[^"]*"/version = "'"$v"'"/m' src-tauri/Cargo.toml
echo "Bumped app/package.json, app/src-tauri/tauri.conf.json, app/src-tauri/Cargo.toml to $v."
echo "Commit and merge, then: git tag app-v$v && git push origin app-v$v"
