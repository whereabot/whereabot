use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use rmcp::{
    ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use serde::Deserialize;

use crate::{AppState, LocateError, authorized, request_location};

#[derive(Deserialize, schemars::JsonSchema)]
struct LocationParams {
    #[schemars(
        range(min = 1, max = 120),
        description = "Seconds to wait for the phone. Default 30."
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
        Parameters(params): Parameters<LocationParams>,
    ) -> CallToolResult {
        let timeout = Duration::from_secs(params.timeout_secs.unwrap_or(30).clamp(1, 120));
        match request_location(&self.state, timeout).await {
            Ok(location) => CallToolResult::success(vec![ContentBlock::text(
                serde_json::to_string(&location).expect("Location serializes"),
            )]),
            Err(LocateError::Offline) => CallToolResult::error(vec![ContentBlock::text(
                "The phone is not connected to the relay. Ask the user to open the Whereabouts app.",
            )]),
            Err(LocateError::Timeout) => CallToolResult::error(vec![ContentBlock::text(
                "The phone did not answer in time.",
            )]),
        }
    }
}

#[tool_handler]
impl ServerHandler for Whereabouts {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }
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
        .layer(middleware::from_fn_with_state(state, authorize))
}

async fn authorize(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    if !authorized(&headers, &state.config.agent_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(request).await
}
