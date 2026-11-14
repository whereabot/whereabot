variable "google_site_verification" {
  description = "TXT value Google Search Console shows for whereabot.io, e.g. google-site-verification=abc..."
  type        = string
}

# Proves domain ownership to Google, so Cloud Run can map relay.whereabot.io.
resource "cloudflare_dns_record" "google_site_verification" {
  zone_id = data.cloudflare_zone.whereabot.zone_id
  name    = "whereabot.io"
  type    = "TXT"
  content = "\"${var.google_site_verification}\""
  ttl     = 1
}

variable "relay_vm_ip" {
  description = "Cutover switch. Empty = Cloud Run (CNAME). The prod relay_vm_ip output = the VM (A record)."
  type        = string
  default     = ""
}

# Cloud Run domain mapping target, or the relay VM at cutover (infra/relay-vm.md). DNS only:
# Google (Cloud Run) or Caddy (VM) issues the certificate, and the Cloudflare proxy would hide
# the record it checks.
resource "cloudflare_dns_record" "relay" {
  zone_id = data.cloudflare_zone.whereabot.zone_id
  name    = "relay.whereabot.io"
  type    = var.relay_vm_ip == "" ? "CNAME" : "A"
  content = var.relay_vm_ip == "" ? "ghs.googlehosted.com" : var.relay_vm_ip
  proxied = false
  ttl     = 1
}
