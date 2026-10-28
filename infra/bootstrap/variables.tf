variable "gcp_project" {
  type    = string
  default = "whereabouts-513001"
}

variable "gcp_region" {
  type    = string
  default = "europe-west1"
}

variable "github_repo" {
  type    = string
  default = "whereabot/whereabot"
}

variable "state_bucket" {
  type    = string
  default = "whereabouts-tfstate"
}
