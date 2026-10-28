use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
};
use tokio::signal::unix::{SignalKind, signal};
use tracing_subscriber::EnvFilter;
use whereabouts_server::{Config, Shutdown, build};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let supabase_url = required("SUPABASE_URL");
    let config = Config {
        issuer: std::env::var("SUPABASE_ISSUER")
            .ok()
            .filter(|issuer| !issuer.is_empty())
            .unwrap_or_else(|| format!("{supabase_url}/auth/v1")),
        public_url: required("WB_PUBLIC_URL"),
        agent_keys: agent_keys_enabled().then(|| required("SUPABASE_SERVICE_ROLE_KEY")),
        supabase_url,
    };
    let addr = bind_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    tracing::info!("whereabouts relay listening on {addr}");
    let (router, shutdown) = build(config);
    let router = router.layer(middleware::from_fn(log_locate_status));
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(shutdown))
        .await
        .expect("serve");
}

/// Reads a required setting. Exits when it is missing or empty.
fn required(key: &str) -> String {
    match std::env::var(key) {
        Ok(value) if !value.is_empty() => value,
        _ => {
            eprintln!("error: {key} must be set");
            std::process::exit(1);
        }
    }
}

/// `WB_AGENT_KEYS=true` (or `1`) keeps the old agent-key path as a headless fallback.
fn agent_keys_enabled() -> bool {
    matches!(std::env::var("WB_AGENT_KEYS").as_deref(), Ok("true" | "1"))
}

/// `WHEREABOUTS_ADDR` wins; Cloud Run sets `PORT`; otherwise bind locally.
fn bind_addr() -> String {
    if let Ok(addr) = std::env::var("WHEREABOUTS_ADDR") {
        return addr;
    }
    match std::env::var("PORT") {
        Ok(port) => format!("0.0.0.0:{port}"),
        Err(_) => "127.0.0.1:8787".into(),
    }
}

async fn log_locate_status(req: Request, next: Next) -> Response {
    let is_locate = req.uri().path() == "/v1/locate";
    let res = next.run(req).await;
    if is_locate {
        tracing::info!(status = res.status().as_u16(), "locate");
    }
    res
}

/// Waits for SIGTERM or Ctrl-C, then tells every phone to reconnect (Close 1012).
async fn shutdown_signal(shutdown: Shutdown) {
    let mut terminate = signal(SignalKind::terminate()).expect("SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
    tracing::info!("shutting down");
    shutdown.close_devices().await;
}
