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
# Lockfiles and the committed iOS plist carry the version too; the IPA check reads CFBundleVersion.
perl -0pi -e 's/("name": "app",\s*"version": )"[^"]*"/$1"'"$v"'"/g' package-lock.json
perl -0pi -e 's/(name = "app"\nversion = )"[^"]*"/$1"'"$v"'"/' src-tauri/Cargo.lock
perl -0pi -e 's#(<key>CFBundle(?:ShortVersionString|Version)</key>\s*<string>)[^<]*#${1}'"$v"'#g' src-tauri/gen/apple/app_iOS/Info.plist
echo "Bumped the app version to $v in manifests, lockfiles and Info.plist."
echo "Commit and merge, then: git tag app-v$v && git push origin app-v$v"
