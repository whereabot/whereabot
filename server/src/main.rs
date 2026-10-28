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
        supabase_url: required("SUPABASE_URL"),
        anon_key: required("SUPABASE_ANON_KEY"),
        service_role_key: required("SUPABASE_SERVICE_ROLE_KEY"),
    };
    let addr = bind_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    tracing::info!("whereabouts relay listening on {addr}");
    let router = app(config).layer(middleware::from_fn(log_locate_status));
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
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

async fn shutdown_signal() {
    let mut terminate = signal(SignalKind::terminate()).expect("SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
    tracing::info!("shutting down");
}
