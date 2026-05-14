//! Shared Graphene WebSocket wire-protocol helpers.
//!
//! These helpers build the Graphene `call(api_id, method, params)` envelope,
//! prepend local callback ids for callback-aware methods, parse pushed notices,
//! and decode WebSocket text frames into JSON values.

use std::net::TcpStream;

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

use crate::RpcError;

use super::types::GrapheneNotice;

pub(crate) fn build_graphene_ws_call_request(
    id: u64,
    api_id: u64,
    method: &str,
    params: Vec<Value>,
) -> Value {
    json!({
        "id": id,
        "method": "call",
        "params": [api_id, method, params],
    })
}

pub(crate) fn callback_params(callback_id: u64, params_after_callback: Vec<Value>) -> Vec<Value> {
    let mut params = Vec::with_capacity(params_after_callback.len() + 1);
    params.push(json!(callback_id));
    params.extend(params_after_callback);
    params
}

pub(super) fn read_websocket_json(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    method: &str,
) -> Result<Value, RpcError> {
    let response = socket
        .read()
        .map_err(|source| RpcError::transport(source.to_string()))?;
    match response {
        Message::Text(text) => serde_json::from_str::<Value>(&text)
            .map_err(|source| RpcError::http(method, source.to_string())),
        other => Err(RpcError::protocol(
            method,
            format!("expected WebSocket text response, got {other:?}"),
        )),
    }
}

pub(crate) fn parse_graphene_notice(response: &Value) -> Result<Option<GrapheneNotice>, RpcError> {
    if response.get("method").and_then(Value::as_str) != Some("notice") {
        return Ok(None);
    }
    let params = response
        .get("params")
        .and_then(Value::as_array)
        .ok_or_else(|| RpcError::protocol("notice", "Graphene notice params must be an array"))?;
    let callback_id = params
        .first()
        .and_then(Value::as_u64)
        .ok_or_else(|| RpcError::protocol("notice", "Graphene notice is missing callback id"))?;
    let payload = params
        .get(1)
        .cloned()
        .ok_or_else(|| RpcError::protocol("notice", "Graphene notice is missing payload"))?;
    Ok(Some(GrapheneNotice {
        callback_id,
        payload,
    }))
}
