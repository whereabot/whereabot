# Relay on an e2-micro VM

The prod relay can run on one Compute Engine e2-micro instead of Cloud Run. Code: [`relay_vm.tf`](modules/whereabouts/relay_vm.tf), [`relay_vm_startup.sh.tftpl`](modules/whereabouts/relay_vm_startup.sh.tftpl), [`relay_vm_variables.tf`](modules/whereabouts/relay_vm_variables.tf). Prod turns it on with `relay_vm_enabled` in [`envs/prod/main.tf`](envs/prod/main.tf).

## Why

Cloud Run with one always-on instance (1 vCPU, 512Mi, CPU always allocated) is about $50 a month at list price. The always-free tier covers one e2-micro in us-central1, us-east1 or us-west1, a 30 GB standard disk and 1 GB of egress a month (North America to the world, China and Australia excluded). One user with 8 devices fits in that.

## How it runs

- Container-Optimized OS. The boot script runs on every boot. It reads the relay secrets from Secret Manager with the VM's service account, starts the relay container on `127.0.0.1:8080`, then starts Caddy.
- Caddy is the only outside listener (ports 80 and 443). It gets and renews the Let's Encrypt certificate for `relay_domain` and keeps it on the disk (`/var/lib/caddy`). It proxies WebSockets with no read or write timeout.
- The VM uses the relay service account that Cloud Run uses, so the secret grants are the same. The env is the same too: [`relay_env.tf`](modules/whereabouts/relay_env.tf).
- Deploy: [`deploy.yml`](../.github/workflows/deploy.yml) builds the image as before. The apply writes the new image into the boot script, then the workflow resets the VM. Phones reconnect after about a minute, as after a Cloud Run revision swap.
- Cloud Run stays up beside the VM. DNS picks which one serves `relay.whereabot.io`, so cutover and rollback are a DNS change.

## One-time setup (human)

1. Re-apply [`bootstrap/`](bootstrap): the deployer gets `roles/compute.admin`, and the relay service accounts get `roles/logging.logWriter`.
2. Merge, then promote to `main`. The prod apply creates the IP, firewall and VM. The `default` network must exist (the Compute API creates it in a new project).
3. Check the relay on the VM before DNS points to it: `gcloud compute ssh whereabouts-relay-prod --zone us-central1-a --command 'curl -sf 127.0.0.1:8080/health'`. Caddy cannot get a certificate yet; that is expected.

## Cutover

1. Get the IP: `tofu -chdir=infra/envs/prod output -raw relay_vm_ip`.
2. In `infra/dns/terraform.tfvars`, set `relay_vm_ip` to it. Apply [`dns/`](dns). The relay record changes from a CNAME to an A record.
3. Verify with a public resolver: `dig +short relay.whereabot.io @1.1.1.1` shows the IP.
4. Reset the VM so Caddy asks for the certificate now, not at its next retry: `gcloud compute instances reset whereabouts-relay-prod --zone us-central1-a`.
5. Wait until `curl -sf https://relay.whereabot.io/health` answers. Then send a real locate request from the app.
6. Change the relay row in [privacy](../docs/privacy.md#services-involved) to Compute Engine.

## Rollback

1. Remove `relay_vm_ip` from `infra/dns/terraform.tfvars` and apply [`dns/`](dns). The record is the Cloud Run CNAME again.
2. Verify with `dig`, then `/health`. The Cloud Run certificate cannot renew while DNS points at the VM, so a rollback within its validity (up to 90 days) is instant; after that, wait for Google to issue a new one.

The VM stays up during a rollback. To remove it, set `relay_vm_enabled = false`; the next prod apply deletes it.

## Cost

- VM, disk and egress up to the free limits: $0. Over 1 GB egress a month: about $0.12 per GB.
- Pulling the image from the registry in `gcp_region` (europe-west1) is cross-region transfer: cents a month.
- Check the first bill for the external IP. Google charges for in-use IPv4 addresses on most VMs.
- Cloud Run still costs money until a later change removes it after the VM has run well.

## Not covered yet

- The WARNING email in [`alerts.tf`](envs/prod/alerts.tf) matches Cloud Run logs only. VM logs are in Cloud Logging under the instance.
- A US region adds round-trip time for users in Europe and Israel.
