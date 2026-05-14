use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

use crate::jsonrpc::parse_json_rpc_response;
use crate::{OpenRpcCallbackParams, OpenRpcParams, RpcError, RpcTransport};

/// Long-lived blocking Graphene WebSocket session.
///
/// Graphene API ids and callback ids are scoped to a single WebSocket
/// connection. This session keeps that scope explicit: API handles returned by
/// `login_api` are valid only for this session. A background dispatcher owns the
/// socket, routes JSON-RPC responses by request id, and routes Graphene notices
/// by callback id.
///
/// Current lifecycle semantics are intentionally simple: the dispatcher does not
/// reconnect. A socket disconnect fails all pending requests, clears callback
/// routing, and causes later calls on the session to return transport errors.
/// Reconnect and resubscribe policy should be designed as a separate layer.
pub struct GrapheneWebSocketSession {
    endpoint: String,
    commands: SyncSender<DispatcherCommand>,
    next_id: AtomicU64,
}

type NoticeCallback = Box<dyn FnMut(Value) + Send + 'static>;

const DEFAULT_WS_COMMAND_QUEUE_CAPACITY: usize = 128;
const DEFAULT_WS_READ_TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiHandle {
    api_id: u64,
}

impl ApiHandle {
    pub fn id(&self) -> u64 {
        self.api_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackHandle {
    callback_id: u64,
}

impl CallbackHandle {
    pub fn id(&self) -> u64 {
        self.callback_id
    }
}

pub struct CallbackSubscription {
    callback_id: u64,
    commands: SyncSender<DispatcherCommand>,
    closed: AtomicBool,
}

impl CallbackSubscription {
    pub fn id(&self) -> u64 {
        self.callback_id
    }

    pub fn handle(&self) -> CallbackHandle {
        CallbackHandle {
            callback_id: self.callback_id,
        }
    }

    pub fn unsubscribe(&self) -> Result<(), RpcError> {
        if self.closed.swap(true, Ordering::Relaxed) {
            return Ok(());
        }
        let (response_tx, response_rx) = mpsc::sync_channel(1);
        self.commands
            .send(DispatcherCommand::Unsubscribe {
                callback_id: self.callback_id,
                response_tx: Some(response_tx),
            })
            .map_err(|_| RpcError::transport("WebSocket dispatcher is not running"))?;
        response_rx
            .recv()
            .map_err(|_| RpcError::transport("WebSocket dispatcher closed unsubscribe response"))?
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }
}

impl Drop for CallbackSubscription {
    fn drop(&mut self) {
        if self.closed.swap(true, Ordering::Relaxed) {
            return;
        }
        let _ = self.commands.send(DispatcherCommand::Unsubscribe {
            callback_id: self.callback_id,
            response_tx: None,
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GrapheneNotice {
    pub callback_id: u64,
    pub payload: Value,
}

pub struct GrapheneWebSocketApiTransport {
    session: Arc<GrapheneWebSocketSession>,
    api: ApiHandle,
}

impl RpcTransport for GrapheneWebSocketApiTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        self.session.call_api_raw(self.api.api_id, method, params)
    }
}

impl GrapheneWebSocketSession {
    pub fn connect(endpoint: impl Into<String>) -> Result<Self, RpcError> {
        let endpoint = endpoint.into();
        let (mut socket, _) =
            connect(endpoint.as_str()).map_err(|source| RpcError::transport(source.to_string()))?;
        set_websocket_read_timeout(&mut socket, Some(DEFAULT_WS_READ_TIMEOUT))?;
        let (commands, command_rx) = mpsc::sync_channel(DEFAULT_WS_COMMAND_QUEUE_CAPACITY);
        let dispatcher_endpoint = endpoint.clone();
        thread::Builder::new()
            .name("graphene-ws-dispatcher".to_owned())
            .spawn(move || run_websocket_dispatcher(dispatcher_endpoint, socket, command_rx))
            .map_err(|source| RpcError::transport(source.to_string()))?;
        Ok(Self {
            endpoint,
            commands,
            next_id: AtomicU64::new(1),
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn api_transport(self: &Arc<Self>, api: &ApiHandle) -> GrapheneWebSocketApiTransport {
        GrapheneWebSocketApiTransport {
            session: Arc::clone(self),
            api: api.clone(),
        }
    }

    pub fn login(&self, user: &str, password: &str) -> Result<Value, RpcError> {
        self.call_api_raw(1, "login", vec![json!(user), json!(password)])
    }

    pub fn login_api(&self, api_name: &str) -> Result<ApiHandle, RpcError> {
        let raw = self.call_api_raw(1, api_name, Vec::new())?;
        let api_id = raw.as_u64().ok_or_else(|| {
            RpcError::protocol(api_name, "login API response should be an API id")
        })?;
        Ok(ApiHandle { api_id })
    }

    pub fn call<P>(&self, api: &ApiHandle, params: P) -> Result<P::Response, RpcError>
    where
        P: OpenRpcParams,
        P::Response: serde::de::DeserializeOwned,
    {
        let raw = self.call_api_raw(api.api_id, P::METHOD, params.into_positional_params())?;
        serde_json::from_value(raw).map_err(|source| RpcError::decode(P::METHOD, source))
    }

    pub fn call_api_raw(
        &self,
        api_id: u64,
        method: &str,
        params: Vec<Value>,
    ) -> Result<Value, RpcError> {
        let request_id = self.next_request_id();
        let request = build_graphene_ws_call_request(request_id, api_id, method, params);
        let (response_tx, response_rx) = mpsc::sync_channel(1);
        self.commands
            .send(DispatcherCommand::Call {
                request_id,
                method: method.to_owned(),
                request,
                response_tx,
            })
            .map_err(|_| RpcError::transport("WebSocket dispatcher is not running"))?;
        response_rx
            .recv()
            .map_err(|_| RpcError::transport("WebSocket dispatcher closed RPC response"))?
    }

    pub fn call_with_callback_raw<F>(
        &self,
        api: &ApiHandle,
        method: &str,
        params_after_callback: Vec<Value>,
        callback: F,
    ) -> Result<(CallbackSubscription, Value), RpcError>
    where
        F: FnMut(Value) + Send + 'static,
    {
        let callback_id = self.next_request_id();
        let request = build_graphene_ws_call_request(
            callback_id,
            api.api_id,
            method,
            callback_params(callback_id, params_after_callback),
        );
        let (response_tx, response_rx) = mpsc::sync_channel(1);
        self.commands
            .send(DispatcherCommand::RegisterCallbackCall {
                request_id: callback_id,
                callback_id,
                method: method.to_owned(),
                request,
                callback: Box::new(callback),
                response_tx,
            })
            .map_err(|_| RpcError::transport("WebSocket dispatcher is not running"))?;
        let response = response_rx
            .recv()
            .map_err(|_| RpcError::transport("WebSocket dispatcher closed callback response"))??;
        Ok((
            CallbackSubscription {
                callback_id,
                commands: self.commands.clone(),
                closed: AtomicBool::new(false),
            },
            response,
        ))
    }

    pub fn call_with_callback_once_raw(
        &self,
        api: &ApiHandle,
        method: &str,
        params_after_callback: Vec<Value>,
    ) -> Result<Value, RpcError> {
        let (payload_tx, payload_rx) = mpsc::sync_channel(1);
        let (subscription, _response) =
            self.call_with_callback_raw(api, method, params_after_callback, move |payload| {
                let _ = payload_tx.send(payload);
            })?;
        let payload = payload_rx
            .recv()
            .map_err(|_| RpcError::transport("WebSocket dispatcher closed callback payload"));
        subscription.unsubscribe()?;
        payload
    }

    /// Register a typed persistent callback RPC.
    ///
    /// Callback payload decode failures are currently dropped because this
    /// ergonomic callback receives decoded notices directly. Future dispatcher
    /// observability should route decode failures to an explicit error handler.
    pub fn subscribe<P, F>(
        &self,
        api: &ApiHandle,
        params: P,
        mut callback: F,
    ) -> Result<(CallbackSubscription, P::Response), RpcError>
    where
        P: OpenRpcCallbackParams,
        F: FnMut(P::Callback) + Send + 'static,
    {
        let (subscription, response) = self.call_with_callback_raw(
            api,
            P::METHOD,
            params.into_positional_params_after_callback(),
            move |payload| {
                if let Ok(decoded) = P::decode_callback(payload) {
                    callback(decoded);
                }
            },
        )?;
        let response = P::decode_response(response)?;
        Ok((subscription, response))
    }

    pub fn call_with_callback_once<P>(
        &self,
        api: &ApiHandle,
        params: P,
    ) -> Result<P::Callback, RpcError>
    where
        P: OpenRpcCallbackParams,
    {
        let raw = self.call_with_callback_once_raw(
            api,
            P::METHOD,
            params.into_positional_params_after_callback(),
        )?;
        P::decode_callback(raw)
    }

    #[deprecated(note = "use call_with_callback_once_raw or typed call_with_callback_once")]
    pub fn call_with_callback_raw_wait(
        &self,
        api: &ApiHandle,
        method: &str,
        params_after_callback: Vec<Value>,
    ) -> Result<Value, RpcError> {
        self.call_with_callback_once_raw(api, method, params_after_callback)
    }

    #[deprecated(
        note = "the dispatcher is now the sole WebSocket reader; register a callback with subscribe/call_with_callback_raw instead"
    )]
    pub fn wait_for_next_notice(&self) -> Result<GrapheneNotice, RpcError> {
        Err(RpcError::protocol(
            "notice",
            "wait_for_next_notice is deprecated after the dispatcher became the sole WebSocket reader; register a callback instead",
        ))
    }

    #[deprecated(note = "use CallbackSubscription::unsubscribe for callback lifecycle cleanup")]
    pub fn remove_callback(&self, handle: &CallbackHandle) {
        let _ = self.commands.send(DispatcherCommand::Unsubscribe {
            callback_id: handle.callback_id,
            response_tx: None,
        });
    }

    pub fn shutdown(&self) -> Result<(), RpcError> {
        let (response_tx, response_rx) = mpsc::sync_channel(1);
        self.commands
            .send(DispatcherCommand::Shutdown { response_tx })
            .map_err(|_| RpcError::transport("WebSocket dispatcher is not running"))?;
        response_rx
            .recv()
            .map_err(|_| RpcError::transport("WebSocket dispatcher closed shutdown response"))?
    }

    fn next_request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

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

enum DispatcherCommand {
    Call {
        request_id: u64,
        method: String,
        request: Value,
        response_tx: SyncSender<Result<Value, RpcError>>,
    },
    RegisterCallbackCall {
        request_id: u64,
        callback_id: u64,
        method: String,
        request: Value,
        callback: NoticeCallback,
        response_tx: SyncSender<Result<Value, RpcError>>,
    },
    Unsubscribe {
        callback_id: u64,
        response_tx: Option<SyncSender<Result<(), RpcError>>>,
    },
    Shutdown {
        response_tx: SyncSender<Result<(), RpcError>>,
    },
}

struct PendingRequest {
    method: String,
    response_tx: SyncSender<Result<Value, RpcError>>,
    cleanup_callback_on_error: Option<u64>,
}

fn run_websocket_dispatcher(
    endpoint: String,
    mut socket: WebSocket<MaybeTlsStream<TcpStream>>,
    commands: Receiver<DispatcherCommand>,
) {
    let mut pending_requests = HashMap::<u64, PendingRequest>::new();
    let mut callbacks = HashMap::<u64, NoticeCallback>::new();

    loop {
        while let Ok(command) = commands.try_recv() {
            if handle_dispatcher_command(
                command,
                &mut socket,
                &mut pending_requests,
                &mut callbacks,
            ) {
                return;
            }
        }

        match read_websocket_json(&mut socket, "websocket_dispatcher") {
            Ok(message) => {
                if let Err(error) =
                    handle_dispatcher_message(message, &mut pending_requests, &mut callbacks)
                {
                    fail_dispatcher(
                        &endpoint,
                        error.to_string(),
                        &mut pending_requests,
                        &mut callbacks,
                    );
                    return;
                }
            }
            Err(RpcError::Transport { message }) if is_timeout_transport_error(&message) => {
                continue;
            }
            Err(error) => {
                fail_dispatcher(
                    &endpoint,
                    error.to_string(),
                    &mut pending_requests,
                    &mut callbacks,
                );
                return;
            }
        }
    }
}

fn handle_dispatcher_command(
    command: DispatcherCommand,
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    pending_requests: &mut HashMap<u64, PendingRequest>,
    callbacks: &mut HashMap<u64, NoticeCallback>,
) -> bool {
    match command {
        DispatcherCommand::Call {
            request_id,
            method,
            request,
            response_tx,
        } => {
            if let Err(error) = send_dispatcher_request(socket, &method, &request) {
                let _ = response_tx.send(Err(error));
                return true;
            }
            pending_requests.insert(
                request_id,
                PendingRequest {
                    method,
                    response_tx,
                    cleanup_callback_on_error: None,
                },
            );
            false
        }
        DispatcherCommand::RegisterCallbackCall {
            request_id,
            callback_id,
            method,
            request,
            callback,
            response_tx,
        } => {
            callbacks.insert(callback_id, callback);
            if let Err(error) = send_dispatcher_request(socket, &method, &request) {
                callbacks.remove(&callback_id);
                let _ = response_tx.send(Err(error));
                return true;
            }
            pending_requests.insert(
                request_id,
                PendingRequest {
                    method,
                    response_tx,
                    cleanup_callback_on_error: Some(callback_id),
                },
            );
            false
        }
        DispatcherCommand::Unsubscribe {
            callback_id,
            response_tx,
        } => {
            callbacks.remove(&callback_id);
            if let Some(response_tx) = response_tx {
                let _ = response_tx.send(Ok(()));
            }
            false
        }
        DispatcherCommand::Shutdown { response_tx } => {
            callbacks.clear();
            fail_pending_requests("WebSocket session shut down", pending_requests);
            let _ = response_tx.send(Ok(()));
            true
        }
    }
}

fn send_dispatcher_request(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    method: &str,
    request: &Value,
) -> Result<(), RpcError> {
    socket
        .send(Message::Text(request.to_string()))
        .map_err(|source| {
            RpcError::transport(format!("WebSocket write failed for {method}: {source}"))
        })
}

fn handle_dispatcher_message(
    message: Value,
    pending_requests: &mut HashMap<u64, PendingRequest>,
    callbacks: &mut HashMap<u64, NoticeCallback>,
) -> Result<(), RpcError> {
    if let Some(notice) = parse_graphene_notice(&message)? {
        if let Some(callback) = callbacks.get_mut(&notice.callback_id) {
            callback(notice.payload);
        }
        return Ok(());
    }

    let response_id = message.get("id").and_then(Value::as_u64).ok_or_else(|| {
        RpcError::protocol(
            "websocket_dispatcher",
            "WebSocket response is missing numeric id",
        )
    })?;

    let Entry::Occupied(entry) = pending_requests.entry(response_id) else {
        return Ok(());
    };
    let pending = entry.remove();
    let result = parse_json_rpc_response(&pending.method, message);
    if result.is_err() {
        if let Some(callback_id) = pending.cleanup_callback_on_error {
            callbacks.remove(&callback_id);
        }
    }
    let _ = pending.response_tx.send(result);
    Ok(())
}

fn fail_dispatcher(
    endpoint: &str,
    reason: String,
    pending_requests: &mut HashMap<u64, PendingRequest>,
    callbacks: &mut HashMap<u64, NoticeCallback>,
) {
    callbacks.clear();
    fail_pending_requests(
        format!("WebSocket session {endpoint} disconnected: {reason}"),
        pending_requests,
    );
}

fn fail_pending_requests(
    reason: impl Into<String>,
    pending_requests: &mut HashMap<u64, PendingRequest>,
) {
    let reason = reason.into();
    for (_, pending) in pending_requests.drain() {
        let _ = pending.response_tx.send(Err(RpcError::transport(format!(
            "{reason}; method={}",
            pending.method
        ))));
    }
}

fn is_timeout_transport_error(message: &str) -> bool {
    message.contains("timed out")
        || message.contains("WouldBlock")
        || message.contains("operation would block")
        || message.contains("temporarily unavailable")
        || message.contains("os error 35")
}

fn set_websocket_read_timeout(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    timeout: Option<Duration>,
) -> Result<(), RpcError> {
    match socket.get_mut() {
        MaybeTlsStream::Plain(stream) => stream.set_read_timeout(timeout),
        MaybeTlsStream::NativeTls(stream) => stream.get_ref().set_read_timeout(timeout),
        _ => Ok(()),
    }
    .map_err(|source| RpcError::transport(source.to_string()))
}

fn read_websocket_json(
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

/// Blocking Graphene WebSocket API transport.
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
