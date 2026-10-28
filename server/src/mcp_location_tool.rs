use std::{sync::Arc, time::Duration};

use axum::{Router, http::request::Parts, middleware};
use rmcp::{
    ServerHandler,
    handler::server::{tool::Extension, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use serde::Deserialize;

use crate::{Agent, AppState, agent_bearer_auth, location_request::request_location};

#[derive(Deserialize, schemars::JsonSchema)]
struct LocationParams {
    #[schemars(
        range(min = 1, max = 120),
        description = "Seconds to wait for the phone, including time for the user to open the app. Default 60."
    )]
    timeout_secs: Option<u64>,
}

#[derive(Clone)]
struct Whereabouts {
    state: Arc<AppState>,
}

#[tool_router]
impl Whereabouts {
    #[tool(
        name = "get_phone_location",
        description = "Ping the user's phone and return its current GPS location (latitude, longitude, accuracy in meters, capture time). Use when you need to know where the user is right now."
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
        let timeout = Duration::from_secs(params.timeout_secs.unwrap_or(60).clamp(1, 120));
        match request_location(&self.state, agent, timeout).await {
            Ok(location) => CallToolResult::success(vec![ContentBlock::text(
                serde_json::to_string(&location).expect("Location serializes"),
            )]),
            Err(error) => CallToolResult::error(vec![ContentBlock::text(error.message())]),
        }
    }
}

#[tool_handler]
impl ServerHandler for Whereabouts {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }
}

/// Tool list as the MCP `tools/list` reply carries it. The docs build prints this.
pub fn tool_list_json() -> serde_json::Value {
    serde_json::to_value(Whereabouts::tool_router().list_all()).expect("tools serialize")
}

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let factory_state = state.clone();
    // Stateless JSON responses: each POST stands alone, so no session header is needed.
    // Host validation is off because the bearer token already gates the route.
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .disable_allowed_hosts();
    let service = StreamableHttpService::new(
        move || {
            Ok(Whereabouts {
                state: factory_state.clone(),
            })
        },
        Arc::new(LocalSessionManager::default()),
        config,
    );
    Router::new()
        .route_service("/", service)
        .layer(middleware::from_fn_with_state(
            state,
            agent_bearer_auth::authorize,
        ))
}
