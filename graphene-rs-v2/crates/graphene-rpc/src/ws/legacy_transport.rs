use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

use crate::jsonrpc::parse_json_rpc_response;
use crate::{RpcError, RpcTransport};

use super::protocol::read_websocket_json;

/// Compatibility transport that opens a fresh socket for each RPC call.
///
/// Prefer `GrapheneWebSocketSession` for new code: Graphene API ids and callback
/// ids are socket-local, and subscriptions require one dispatcher-owned socket.
/// This transport remains useful for simple one-shot database or broadcast calls
/// that do not need callback routing.
///
/// Unlike direct HTTP database endpoints, Graphene WebSocket APIs are normally
/// invoked as `call(api_id, method, params)`. Use API id `0` for database API
/// and the id returned by login API's `network_broadcast` method for broadcast.
#[derive(Debug)]
pub struct GrapheneWebSocketTransport {
    endpoint: String,
    api: GrapheneWebSocketApi,
    next_id: AtomicU64,
}

#[derive(Clone, Debug)]
enum GrapheneWebSocketApi {
    Id(u64),
    LoginMethod(String),
}

impl GrapheneWebSocketTransport {
    pub fn new(endpoint: impl Into<String>, api_id: u64) -> Self {
        Self {
            endpoint: endpoint.into(),
            api: GrapheneWebSocketApi::Id(api_id),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn login_api(endpoint: impl Into<String>, login_method: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            api: GrapheneWebSocketApi::LoginMethod(login_method.into()),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn next_request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

impl RpcTransport for GrapheneWebSocketTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        let (mut socket, _) = connect(self.endpoint.as_str())
            .map_err(|source| RpcError::transport(source.to_string()))?;
        let api_id = match &self.api {
            GrapheneWebSocketApi::Id(api_id) => *api_id,
            GrapheneWebSocketApi::LoginMethod(login_method) => {
                let request = json!({
                    "id": self.next_request_id(),
                    "method": "call",
                    "params": [1, login_method, []],
                });
                websocket_json_rpc_call(&mut socket, login_method, request)?
                    .as_u64()
                    .ok_or_else(|| {
                        RpcError::protocol(login_method, "login API response should be an API id")
                    })?
            }
        };
        let request = json!({
            "id": self.next_request_id(),
            "method": "call",
            "params": [api_id, method, params],
        });
        websocket_json_rpc_call(&mut socket, method, request)
    }
}

fn websocket_json_rpc_call(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    method: &str,
    request: Value,
) -> Result<Value, RpcError> {
    socket
        .send(Message::Text(request.to_string()))
        .map_err(|source| RpcError::transport(source.to_string()))?;
    let response = read_websocket_json(socket, method)?;
    parse_json_rpc_response(method, response)
}
