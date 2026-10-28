# DNS for whereabot.io (Cloudflare zone in the "whereabot" account). Applied by a human,
# not by CI, so deploy.yml needs no Cloudflare credentials. State: same bucket, own prefix.
# The apex record is not here: `wrangler deploy --domains whereabot.io` creates and owns it.
terraform {
  required_version = ">= 1.6"
  backend "gcs" {
    bucket = "whereabouts-tfstate"
    prefix = "whereabouts/dns"
  }
  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5.0"
    }
  }
}

# Token: CLOUDFLARE_API_TOKEN env var, scoped to Zone DNS: Edit on whereabot.io.
provider "cloudflare" {}

data "cloudflare_zone" "whereabot" {
  filter = {
    name = "whereabot.io"
  }
}
