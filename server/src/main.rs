use whereabouts_server::{Config, app};

#[tokio::main]
async fn main() {
    let env = |key: &str| std::env::var(key).unwrap_or_else(|_| panic!("{key} is required"));
    let config = Config {
        agent_token: env("WHEREABOUTS_AGENT_TOKEN"),
        device_token: env("WHEREABOUTS_DEVICE_TOKEN"),
    };
    let addr = std::env::var("WHEREABOUTS_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    println!("whereabouts relay listening on {addr}");
    axum::serve(listener, app(config)).await.expect("serve");
}
