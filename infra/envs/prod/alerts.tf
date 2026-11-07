# Emails the owner when the relay itself logs a WARNING or worse: a failed locate request, a late
# phone answer, or any other fault (docs/operations.md#find-failed-locate-requests).
variable "alert_email" {
  description = "Where relay alerts go. deploy.yml sets TF_VAR_alert_email from the ALERT_EMAIL variable; empty = no alerts."
  type        = string
  default     = ""
}

resource "google_project_service" "monitoring" {
  service            = "monitoring.googleapis.com"
  disable_on_destroy = false
}

resource "google_monitoring_notification_channel" "owner" {
  count        = var.alert_email == "" ? 0 : 1
  display_name = "Whereabot owner"
  type         = "email"
  labels       = { email_address = var.alert_email }
  depends_on   = [google_project_service.monitoring]
}

resource "google_monitoring_alert_policy" "relay_warning" {
  count                 = length(google_monitoring_notification_channel.owner)
  display_name          = "Whereabot relay: a locate failed or the relay logged a warning"
  combiner              = "OR"
  notification_channels = [google_monitoring_notification_channel.owner[0].id]
  conditions {
    display_name = "relay log at WARNING or worse"
    condition_matched_log {
      # Only the relay's own lines: Cloud Run's request log marks every 4xx (the normal 401
      # OAuth challenge on /mcp) as WARNING, which alerted on healthy traffic.
      filter = "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"whereabouts-relay-prod\" AND log_id(\"run.googleapis.com/stdout\") AND severity>=WARNING"
    }
  }
  # One email per 5 minutes at most, so a burst of failures does not flood the inbox.
  alert_strategy {
    notification_rate_limit {
      period = "300s"
    }
    auto_close = "1800s"
  }
  documentation {
    content   = "Open the log entry, then filter the relay logs on its jsonPayload.request_id. Runbook: docs/operations.md, \"Find failed locate requests\"."
    mime_type = "text/markdown"
  }
}
