//! Whose phones one request rings: the agent's own user, or a person who shares with them.

use std::time::Duration;

use uuid::Uuid;

use crate::{
    Agent, AppState,
    location_request::request_location,
    location_request_error::LocateError,
    relay_wire_protocol::{Location, Requester},
    shared_person_resolution,
};

/// `requester` is set when someone else's agent asks; `name` then fills the denied text.
pub struct Target {
    pub owner: Uuid,
    pub email: Option<String>,
    pub name: Option<String>,
    pub requester: Option<Requester>,
}

impl Target {
    pub fn own(agent: &Agent) -> Self {
        Self {
            owner: agent.user_id,
            email: agent.email.clone(),
            name: None,
            requester: None,
        }
    }
}

/// Locates the agent's own phones when `person` is absent or blank, else that person's.
pub async fn locate_person(
    state: &AppState,
    agent: &Agent,
    person: Option<&str>,
    timeout: Duration,
) -> Result<Location, LocateError> {
    let target = match person.filter(|person| !person.trim().is_empty()) {
        None => Target::own(agent),
        Some(person) => shared_person_resolution::resolve(state, agent, person).await?,
    };
    request_location(state, agent, target, timeout).await
}
