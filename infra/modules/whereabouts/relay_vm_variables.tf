# The relay on a Compute Engine e2-micro (relay_vm.tf), in the always-free tier. Cloud Run stays
# up beside it; DNS (infra/dns/) picks which one serves the relay host.
variable "relay_vm_enabled" {
  description = "Run the relay on an e2-micro VM with Caddy for TLS. Needs relay_domain."
  type        = bool
  default     = false
}

variable "relay_vm_zone" {
  description = "VM zone. The free tier covers one e2-micro in us-central1, us-east1 or us-west1 only."
  type        = string
  default     = "us-central1-a"
}
