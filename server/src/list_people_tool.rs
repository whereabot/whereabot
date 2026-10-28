//! MCP tool `list_people_sharing_with_me`: who the agent may name in `person`.

use axum::http::request::Parts;
use rmcp::{
    handler::server::tool::Extension,
    model::{CallToolResult, ContentBlock},
    tool, tool_router,
};

use crate::{Agent, mcp_location_tool::Whereabouts, shared_person_resolution};

#[tool_router(router = list_people_router, vis = "pub(crate)")]
impl Whereabouts {
    #[tool(
        name = "list_people_sharing_with_me",
        description = "List the people who share their location with you: display_name and email. Pass either as `person` to get_phone_location."
    )]
    async fn list_people_sharing_with_me(
        &self,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let Some(agent) = parts.extensions.get::<Agent>() else {
            return CallToolResult::error(vec![ContentBlock::text("Unauthorized.")]);
        };
        match shared_person_resolution::people(&self.state, agent).await {
            Some(people) => CallToolResult::success(vec![ContentBlock::text(
                serde_json::to_string(&people).expect("people serialize"),
            )]),
            None => CallToolResult::error(vec![ContentBlock::text(
                "Could not read who shares with you. Try again in a moment.",
            )]),
        }
    }
}
