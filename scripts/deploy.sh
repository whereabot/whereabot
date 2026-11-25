#!/usr/bin/env bash
# BREAK-GLASS ONLY. Normal deploys: .github/workflows/deploy.yml (see infra/README.md).
# Usage: scripts/deploy.sh <dev|prod>
set -euo pipefail

ENV="${1:-}"
case "$ENV" in
  dev|prod) ;;
  *) echo "usage: $0 <dev|prod>" >&2; exit 2 ;;
esac

for tool in tofu docker git gcloud; do
  command -v "$tool" >/dev/null || { echo "missing tool: $tool" >&2; exit 1; }
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SHA="$(git -C "$ROOT" rev-parse --short HEAD)"
TF_DIR="$ROOT/infra/envs/$ENV"

# Repo prefix comes from the state, so it matches the applied registry.
REPO="$(tofu -chdir="$TF_DIR" output -raw artifact_repository)"
IMAGE="$REPO:$SHA"

gcloud auth configure-docker "${REPO%%/*}" --quiet >/dev/null
docker buildx build --platform linux/amd64 --tag "$IMAGE" --push "$ROOT/server"

tofu -chdir="$TF_DIR" apply -var "image=$IMAGE"

# The relay VM (infra/relay-vm.md) reads its image only at boot, so reset it to run the new one.
# ponytail: resets even when the image tag did not change; deploy.yml checks the plan instead.
VM="$(tofu -chdir="$TF_DIR" output -raw relay_vm 2>/dev/null || true)"
if [ -n "$VM" ]; then
  gcloud compute instances reset "$VM" --quiet
  echo "reset relay VM; check its serial log for: whereabouts-relay started $IMAGE"
fi

# Migrations deploy via .github/workflows/migrations.yml, not here.
echo "deployed $ENV at $SHA"
