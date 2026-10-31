use std::{sync::Arc, time::Duration};

use axum::http::request::Parts;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Extension, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;

use crate::{Agent, AppState, location_target::locate_person};

/// MCP clients (Claude Code among them) abort a tool call after 60 s. A relay that waits the
/// full 60 s answers after the abort, and the agent sees a bare "timed out" instead of why.
const MAX_WAIT_SECS: u64 = 55;

#[derive(Deserialize, schemars::JsonSchema)]
struct LocationParams {
    #[schemars(
        range(min = 1, max = 55),
        description = "Seconds to wait for the phone, including time for the user to open the app. Default and maximum 55, so the answer arrives before MCP clients abort a call at 60 s."
    )]
    timeout_secs: Option<u64>,
    #[schemars(
        description = "Email or display name of someone who shares their location with you (see list_people_sharing_with_me). Omit for your own phones."
    )]
    person: Option<String>,
}

#[derive(Clone)]
pub struct Whereabouts {
    pub state: Arc<AppState>,
}

#[tool_router]
impl Whereabouts {
    #[tool(
        name = "get_phone_location",
        title = "Get phone location",
        // Reads only, but every call pings a person's phone again: not idempotent, open world.
        annotations(
            title = "Get phone location",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        ),
        description = "Ping the user's phone and return its current GPS location (latitude, longitude, accuracy in meters, capture time). Use when you need to know where the user is right now. Pass `person` to locate someone who shares their location with the user."
    )]
    async fn get_phone_location(
        &self,
        // rmcp copies the HTTP request parts into the request context; the middleware put Agent inside them.
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<LocationParams>,
    ) -> CallToolResult {
        let Some(agent) = parts.extensions.get::<Agent>() else {
            return CallToolResult::error(vec![ContentBlock::text("Unauthorized.")]);
        };
        let timeout = Duration::from_secs(
            params
                .timeout_secs
                .unwrap_or(MAX_WAIT_SECS)
                .clamp(1, MAX_WAIT_SECS),
        );
        match locate_person(&self.state, agent, params.person.as_deref(), timeout).await {
            Ok(location) => CallToolResult::success(vec![ContentBlock::text(
                serde_json::to_string(&location).expect("Location serializes"),
            )]),
            Err(error) => CallToolResult::error(vec![ContentBlock::text(error.message())]),
        }
    }
    /// Every tool: this file's and `list_people_tool`'s.
    fn tools() -> ToolRouter<Self> {
        Self::tool_router() + Self::list_people_router()
    }
}

#[tool_handler(router = Self::tools())]
impl ServerHandler for Whereabouts {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }
}

/// Tool list as the MCP `tools/list` reply carries it. The docs build prints this.
pub fn tool_list_json() -> serde_json::Value {
    serde_json::to_value(Whereabouts::tools().list_all()).expect("tools serialize")
}
