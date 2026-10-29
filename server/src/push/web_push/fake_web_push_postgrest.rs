//! PostgREST side of the Web Push double: own subscriptions `live` and `gone`; the share RPC
//! gives `OWNER`'s `owner` (no id).

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{RawQuery, State},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use serde_json::{Value, json};
use web_push_native::p256::elliptic_curve::sec1::ToEncodedPoint;

use super::fake_web_push_service::{FakeWeb, Shared};
use crate::push::fake_share_rpcs::OWNER;

pub const GONE_ID: &str = "00000000-0000-4000-8000-00000000006e";
pub const AUTH: [u8; 16] = [9; 16];

fn sub(f: &FakeWeb, name: &str, id: Option<&str>) -> Value {
    let key = f.ua.public_key().to_encoded_point(false);
    json!({
        "id": id,
        "endpoint": format!("{}/push/{name}", f.url.get().unwrap()),
        "p256dh": B64.encode(key.as_bytes()),
        "auth": B64.encode(AUTH),
    })
}

async fn own(State(f): Shared) -> Json<Value> {
    let live = "00000000-0000-4000-8000-000000000011";
    Json(json!([
        sub(&f, "live", Some(live)),
        sub(&f, "gone", Some(GONE_ID))
    ]))
}

async fn shared(State(f): Shared, Json(body): Json<Value>) -> Json<Value> {
    let owner = body["p_owner"] == OWNER;
    Json(json!(if owner {
        vec![sub(&f, "owner", None)]
    } else {
        vec![]
    }))
}

async fn drop_sub(State(f): Shared, RawQuery(q): RawQuery) {
    f.deleted.lock().unwrap().push(q.unwrap_or_default());
}

pub fn routes() -> Router<Arc<FakeWeb>> {
    Router::new()
        .route("/rest/v1/web_push_subscriptions", get(own).delete(drop_sub))
        .route(
            "/rest/v1/rpc/share_target_web_push_subscriptions",
            post(shared),
        )
}
