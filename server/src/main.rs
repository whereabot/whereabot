use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
};
use tokio::signal::unix::{SignalKind, signal};
use tracing_subscriber::EnvFilter;
use whereabouts_server::{Config, app};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let config = Config {
        agent_token: token("WHEREABOUTS_AGENT_TOKEN"),
        device_token: token("WHEREABOUTS_DEVICE_TOKEN"),
    };
    let addr = std::env::var("WHEREABOUTS_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    tracing::info!("whereabouts relay listening on {addr}");
    let router = app(config).layer(middleware::from_fn(log_locate_status));
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve");
}

/// Reads a token from the environment. Exits when it is missing or too short.
fn token(key: &str) -> String {
    let value = std::env::var(key).unwrap_or_default();
    if value.len() < 32 {
        eprintln!(
            "error: {key} must be set and at least 32 characters (try: openssl rand -hex 32)"
        );
        std::process::exit(1);
    }
    value
}

async fn log_locate_status(req: Request, next: Next) -> Response {
    let is_locate = req.uri().path() == "/v1/locate";
    let res = next.run(req).await;
    if is_locate {
        tracing::info!(status = res.status().as_u16(), "locate");
    }
    res
}

async fn shutdown_signal() {
    let mut terminate = signal(SignalKind::terminate()).expect("SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
    tracing::info!("shutting down");
}
