use uuid::Uuid;

/// An authenticated agent request: whose phone to ping, and who asks. Set by the MCP auth middleware.
#[derive(Clone)]
pub struct Agent {
    pub user_id: Uuid,
    pub client_id: String,
    /// From the token's `email` claim; names the account in the offline message.
    pub email: Option<String>,
    /// The agent's Supabase user JWT; reads that user's push tokens under RLS. None for agent keys.
    pub bearer: Option<String>,
}
