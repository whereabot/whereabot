//! The `/mcp` route: streamable HTTP transport for the Whereabouts tools, behind bearer auth.

use std::sync::Arc;

use axum::{Router, middleware};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

use crate::{AppState, agent_bearer_auth, mcp_location_tool::Whereabouts};

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
