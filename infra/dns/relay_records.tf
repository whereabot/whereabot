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

# Cloud Run domain mapping target. DNS only: Google issues the certificate and the
# Cloudflare proxy would hide the CNAME it checks.
resource "cloudflare_dns_record" "relay" {
  zone_id = data.cloudflare_zone.whereabot.zone_id
  name    = "relay.whereabot.io"
  type    = "CNAME"
  content = "ghs.googlehosted.com"
  proxied = false
  ttl     = 1
}
