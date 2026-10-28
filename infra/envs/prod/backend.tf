# Bucket created by the bootstrap step in infra/README.md.
terraform {
  backend "gcs" {
    bucket = "whereabouts-tfstate"
    prefix = "whereabouts/prod"
  }
}
