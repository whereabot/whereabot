variable "apple_client_secret" {
  description = "Sign in with Apple client secret JWT. deploy.yml sets TF_VAR_apple_client_secret; empty = Apple sign-in off."
  type        = string
  sensitive   = true
  default     = ""
}
