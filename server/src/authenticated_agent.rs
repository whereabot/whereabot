use uuid::Uuid;

use crate::relay_wire_protocol::Requester;

/// An authenticated agent request: whose phone to ping, and who asks. Set by the MCP auth middleware.
#[derive(Clone)]
pub struct Agent {
    pub user_id: Uuid,
    pub client_id: String,
    /// From the token's `email` claim; names the account in the offline message.
    pub email: Option<String>,
    /// From the token's `user_metadata`; names the requester on someone else's phone.
    pub name: Option<String>,
    /// The agent's Supabase user JWT; reads that user's push tokens under RLS. None for agent keys.
    pub bearer: Option<String>,
}

impl Agent {
    /// This agent's user as the owner's phone shows them: name, else email, else "Someone".
    pub fn requester(&self) -> Requester {
        let name = self.name.clone().or_else(|| self.email.clone());
        Requester {
            user_id: self.user_id,
            name: name.unwrap_or_else(|| "Someone".into()),
            email: self.email.clone(),
        }
    }
}
