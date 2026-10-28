#![allow(dead_code)]
//! Fake phone whose reply to each ping the test scripts.

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use super::phone::connect_silent_phone;

/// Connects a phone that sends `hello` first, answers each ping with `reply(ping)`
/// (`None` = stay silent), and forwards every ping it gets to the returned channel.
pub async fn connect_scripted(
    addr: &str,
    token: &str,
    hello: Option<Value>,
    reply: impl Fn(&Value) -> Option<Value> + Send + 'static,
) -> mpsc::UnboundedReceiver<Value> {
    let mut ws = connect_silent_phone(addr, token).await;
    if let Some(hello) = hello {
        ws.send(Message::Text(hello.to_string().into()))
            .await
            .unwrap();
    }
    let (pings, seen) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws.next().await {
            let Message::Text(text) = msg else { continue };
            let ping: Value = serde_json::from_str(&text).unwrap();
            let answer = reply(&ping);
            let _ = pings.send(ping); // a test that ignores the pings drops the receiver
            if let Some(answer) = answer {
                ws.send(Message::Text(answer.to_string().into()))
                    .await
                    .unwrap();
            }
        }
    });
    seen
}
