use serde::Deserialize;
use uuid::Uuid;

use crate::{AppState, supabase};

#[derive(Deserialize)]
struct Row {
    revoked_at: Option<String>,
}

/// True when the user removed this device. Reads the row with the phone's own bearer, so
/// RLS limits it to that user. Unreachable Supabase or a missing row counts as not revoked.
pub async fn is_revoked(state: &AppState, token: &str, device: Uuid) -> bool {
    let res = supabase::http()
        .get(format!(
            "{}/rest/v1/devices?id=eq.{device}&select=revoked_at",
            state.supabase_url
        ))
        .header("apikey", state.anon_key.as_deref().unwrap_or(token))
        .bearer_auth(token)
        .send()
        .await;
    let rows = supabase::call::<Vec<Row>>("device_revoked", res).await;
    rows.is_some_and(|rows| rows.iter().any(|row| row.revoked_at.is_some()))
}
