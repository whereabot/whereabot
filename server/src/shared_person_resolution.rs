//! Who shares their location with the agent's user, and which of them `person` names.
//! Read from `location_shares` with the agent's own JWT: RLS shows a grantee only its shares.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    Agent, AppState, location_request_error::LocateError, location_target::Target,
    supabase_rest_client,
};

/// One active share where the agent's user is the grantee.
#[derive(Deserialize, Serialize, Clone)]
pub struct Share {
    #[serde(skip_serializing)]
    owner_user_id: Uuid,
    #[serde(rename(serialize = "display_name"))]
    owner_display_name: Option<String>,
    #[serde(rename(serialize = "email"))]
    owner_email: Option<String>,
}

/// None when the lookup fails, or for agent keys (no user JWT to read with).
pub async fn people(state: &AppState, agent: &Agent) -> Option<Vec<Share>> {
    let bearer = agent.bearer.as_deref()?;
    let res = supabase_rest_client::http()
        .get(format!(
            "{}/rest/v1/location_shares?select=owner_user_id,owner_display_name,owner_email&revoked_at=is.null",
            state.supabase_url
        ))
        .header("apikey", state.anon_key.as_deref().unwrap_or(bearer))
        .bearer_auth(bearer)
        .send()
        .await;
    let shares = supabase_rest_client::call::<Vec<Share>>("location_shares select", res).await?;
    // Defense in depth: a share of your own never makes you your own requester.
    Some(
        shares
            .into_iter()
            .filter(|s| s.owner_user_id != agent.user_id)
            .collect(),
    )
}

/// The one owner whose email or display name equals `person`, ignoring case.
pub async fn resolve(state: &AppState, agent: &Agent, person: &str) -> Result<Target, LocateError> {
    let person = person.trim();
    let not_shared = || LocateError::NotShared(person.to_owned());
    let wanted = person.to_lowercase();
    let same = |value: &Option<String>| {
        value
            .as_deref()
            .is_some_and(|v| v.trim().to_lowercase() == wanted)
    };
    let mut found: Vec<Share> = Vec::new();
    for share in people(state, agent).await.ok_or_else(not_shared)? {
        let matches = same(&share.owner_email) || same(&share.owner_display_name);
        if matches && !found.iter().any(|f| f.owner_user_id == share.owner_user_id) {
            found.push(share);
        }
    }
    match found.len() {
        0 => Err(not_shared()),
        1 => {
            let share = found.remove(0);
            Ok(Target {
                owner: share.owner_user_id,
                name: share
                    .owner_display_name
                    .or_else(|| share.owner_email.clone()),
                email: share.owner_email,
                requester: Some(agent.requester()),
            })
        }
        _ => Err(LocateError::Ambiguous {
            person: person.to_owned(),
            matches: found.iter().map(label).collect(),
        }),
    }
}

/// "Name (email)", or whichever of the two exists.
fn label(share: &Share) -> String {
    match (&share.owner_display_name, &share.owner_email) {
        (Some(name), Some(email)) => format!("{name} ({email})"),
        (name, email) => name.clone().or(email.clone()).unwrap_or_default(),
    }
}
