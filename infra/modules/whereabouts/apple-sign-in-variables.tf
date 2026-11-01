variable "apple_services_id" {
  description = "Apple Services ID for web Sign in with Apple. Its return URL is <supabase_url>/auth/v1/callback."
  type        = string
  default     = "io.whereabot.signin"
}

variable "apple_client_secret" {
  description = "Client secret JWT for the Services ID (ES256, from the .p8 key, max 6 months). Empty = Apple sign-in off."
  type        = string
  sensitive   = true
  default     = ""
}
