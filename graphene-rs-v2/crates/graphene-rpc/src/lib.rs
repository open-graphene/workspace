use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GrapheneUInt64(u64);

impl GrapheneUInt64 {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for GrapheneUInt64 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl From<GrapheneUInt64> for u64 {
    fn from(value: GrapheneUInt64) -> Self {
        value.0
    }
}

impl From<u64> for GrapheneUInt64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl FromStr for GrapheneUInt64 {
    type Err = std::num::ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse::<u64>().map(Self)
    }
}

impl TryFrom<&str> for GrapheneUInt64 {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for GrapheneUInt64 {
    type Error = std::num::ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Serialize for GrapheneUInt64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for GrapheneUInt64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = GrapheneUInt64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a u64 number or decimal string")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(GrapheneUInt64(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GrapheneInt64(i64);

impl GrapheneInt64 {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl fmt::Display for GrapheneInt64 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl From<GrapheneInt64> for i64 {
    fn from(value: GrapheneInt64) -> Self {
        value.0
    }
}

impl From<i64> for GrapheneInt64 {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl FromStr for GrapheneInt64 {
    type Err = std::num::ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse::<i64>().map(Self)
    }
}

impl TryFrom<&str> for GrapheneInt64 {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for GrapheneInt64 {
    type Error = std::num::ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Serialize for GrapheneInt64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.0)
    }
}

impl<'de> Deserialize<'de> for GrapheneInt64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = GrapheneInt64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an i64 number or decimal string")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(GrapheneInt64(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i64::try_from(value).map(GrapheneInt64).map_err(E::custom)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Graphene wire-format timestamp.
///
/// Graphene APIs serialize `fc::time_point_sec` values as UTC timestamps without
/// an explicit timezone suffix, for example `2026-05-13T14:44:57`. This is not
/// RFC 3339 `date-time`, so generated chain bindings use this scalar instead of
/// `chrono::DateTime<Utc>`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GrapheneTimePointSec(NaiveDateTime);

impl GrapheneTimePointSec {
    pub const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S";

    pub fn new(value: NaiveDateTime) -> Self {
        Self(value)
    }

    pub fn naive_utc(self) -> NaiveDateTime {
        self.0
    }
}

impl fmt::Display for GrapheneTimePointSec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.format(Self::FORMAT))
    }
}

impl FromStr for GrapheneTimePointSec {
    type Err = chrono::ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        NaiveDateTime::parse_from_str(value, Self::FORMAT).map(Self)
    }
}

impl TryFrom<&str> for GrapheneTimePointSec {
    type Error = chrono::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for GrapheneTimePointSec {
    type Error = chrono::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<GrapheneTimePointSec> for String {
    fn from(value: GrapheneTimePointSec) -> Self {
        value.to_string()
    }
}

impl Serialize for GrapheneTimePointSec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GrapheneTimePointSec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&value, Self::FORMAT)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

/// Generated OpenRPC parameter structs implement this trait.
///
/// The trait is hand-written runtime API. Chain crates generate only the
/// method-specific structs and impls.
pub trait OpenRpcParams {
    const METHOD: &'static str;
    type Response;

    fn into_positional_params(self) -> Vec<Value>;
}

/// Minimal transport abstraction for typed JSON-RPC calls.
///
/// Concrete HTTP/WebSocket transports can be added without changing the
/// generated chain bindings.
pub trait RpcTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError>;
}

#[derive(Debug)]
pub struct RpcClient<T> {
    transport: T,
}

impl<T> RpcClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn into_transport(self) -> T {
        self.transport
    }
}

impl<T> RpcClient<T>
where
    T: RpcTransport,
{
    pub fn call<P>(&self, params: P) -> Result<P::Response, RpcError>
    where
        P: OpenRpcParams,
        P::Response: serde::de::DeserializeOwned,
    {
        let raw = self
            .transport
            .call_raw(P::METHOD, params.into_positional_params())?;
        serde_json::from_value(raw).map_err(|source| RpcError::decode(P::METHOD, source))
    }
}

/// Blocking HTTP JSON-RPC transport.
///
/// This is intentionally small: it handles JSON-RPC envelopes and delegates all
/// chain-specific typing to generated `OpenRpcParams` implementations.
#[derive(Debug)]
pub struct HttpTransport {
    endpoint: String,
    next_id: AtomicU64,
}

impl HttpTransport {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
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

impl RpcTransport for HttpTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        let request = build_json_rpc_request(self.next_request_id(), method, params);
        let response = ureq::post(&self.endpoint)
            .set("content-type", "application/json")
            .send_json(request)
            .map_err(|source| http_error_with_body(method, source))?;
        let response = response
            .into_json::<Value>()
            .map_err(|source| RpcError::http(method, source.to_string()))?;
        parse_json_rpc_response(method, response)
    }
}

/// Long-lived blocking Graphene WebSocket session.
///
/// Graphene API ids and callback ids are scoped to a single WebSocket
/// connection. This session keeps that scope explicit: API handles returned by
/// `login_api` are valid only for this session. A background dispatcher owns the
/// socket, routes JSON-RPC responses by request id, and routes Graphene notices
/// by callback id.
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

    pub fn call_with_callback_raw_wait(
        &self,
        api: &ApiHandle,
        method: &str,
        params_after_callback: Vec<Value>,
    ) -> Result<Value, RpcError> {
        self.call_with_callback_once_raw(api, method, params_after_callback)
    }

    pub fn wait_for_next_notice(&self) -> Result<GrapheneNotice, RpcError> {
        Err(RpcError::protocol(
            "notice",
            "wait_for_next_notice is deprecated after the dispatcher became the sole WebSocket reader; register a callback instead",
        ))
    }

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

fn build_graphene_ws_call_request(id: u64, api_id: u64, method: &str, params: Vec<Value>) -> Value {
    json!({
        "id": id,
        "method": "call",
        "params": [api_id, method, params],
    })
}

fn callback_params(callback_id: u64, params_after_callback: Vec<Value>) -> Vec<Value> {
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

fn parse_graphene_notice(response: &Value) -> Result<Option<GrapheneNotice>, RpcError> {
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

fn http_error_with_body(method: &str, source: ureq::Error) -> RpcError {
    match source {
        ureq::Error::Status(code, response) => {
            let body = response
                .into_string()
                .unwrap_or_else(|error| format!("<failed to read response body: {error}>"));
            RpcError::http(method, format!("status code {code}: {body}"))
        }
        other => RpcError::http(method, other.to_string()),
    }
}

fn build_json_rpc_request(id: u64, method: &str, params: Vec<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    })
}

fn parse_json_rpc_response(method: &str, response: Value) -> Result<Value, RpcError> {
    let Some(object) = response.as_object() else {
        return Err(RpcError::protocol(
            method,
            "JSON-RPC response must be an object",
        ));
    };

    if let Some(error) = object.get("error") {
        let Some(error_object) = error.as_object() else {
            return Err(RpcError::protocol(
                method,
                "JSON-RPC error must be an object",
            ));
        };
        let code = error_object
            .get("code")
            .and_then(Value::as_i64)
            .ok_or_else(|| RpcError::protocol(method, "JSON-RPC error is missing numeric code"))?;
        let message = error_object
            .get("message")
            .and_then(Value::as_str)
            .ok_or_else(|| RpcError::protocol(method, "JSON-RPC error is missing message"))?
            .to_owned();
        return Err(RpcError::json_rpc(
            method,
            code,
            message,
            error_object.get("data").cloned(),
        ));
    }

    object
        .get("result")
        .cloned()
        .ok_or_else(|| RpcError::protocol(method, "JSON-RPC response is missing result"))
}

#[derive(Debug)]
pub enum RpcError {
    Transport {
        message: String,
    },
    Http {
        method: String,
        message: String,
    },
    JsonRpc {
        method: String,
        code: i64,
        message: String,
        data: Option<Value>,
    },
    Protocol {
        method: String,
        message: String,
    },
    Decode {
        method: &'static str,
        source: serde_json::Error,
    },
}

impl RpcError {
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
        }
    }

    pub fn http(method: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Http {
            method: method.into(),
            message: message.into(),
        }
    }

    pub fn json_rpc(
        method: impl Into<String>,
        code: i64,
        message: impl Into<String>,
        data: Option<Value>,
    ) -> Self {
        Self::JsonRpc {
            method: method.into(),
            code,
            message: message.into(),
            data,
        }
    }

    pub fn protocol(method: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Protocol {
            method: method.into(),
            message: message.into(),
        }
    }

    pub fn decode(method: &'static str, source: serde_json::Error) -> Self {
        Self::Decode { method, source }
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport { message } => write!(formatter, "RPC transport error: {message}"),
            Self::Http { method, message } => {
                write!(formatter, "HTTP RPC error for {method}: {message}")
            }
            Self::JsonRpc {
                method,
                code,
                message,
                data,
            } => {
                if let Some(data) = data {
                    write!(
                        formatter,
                        "JSON-RPC error for {method}: {code} {message}; data={data}"
                    )
                } else {
                    write!(formatter, "JSON-RPC error for {method}: {code} {message}")
                }
            }
            Self::Protocol { method, message } => {
                write!(formatter, "JSON-RPC protocol error for {method}: {message}")
            }
            Self::Decode { method, source } => {
                write!(
                    formatter,
                    "failed to decode RPC response for {method}: {source}"
                )
            }
        }
    }
}

impl Error for RpcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode { source, .. } => Some(source),
            Self::Transport { .. }
            | Self::Http { .. }
            | Self::JsonRpc { .. }
            | Self::Protocol { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_graphene_ws_call_request, build_json_rpc_request, callback_params,
        parse_graphene_notice, parse_json_rpc_response, GrapheneInt64, GrapheneNotice,
        GrapheneTimePointSec, GrapheneUInt64, HttpTransport, OpenRpcParams, RpcClient, RpcError,
        RpcTransport,
    };
    use chrono::NaiveDate;
    use serde::Deserialize;
    use serde_json::{json, Value};
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};

    #[derive(Debug, Deserialize, PartialEq)]
    struct Pong {
        ok: bool,
    }

    struct PingParams;

    impl OpenRpcParams for PingParams {
        const METHOD: &'static str = "ping";
        type Response = Pong;

        fn into_positional_params(self) -> Vec<Value> {
            vec![json!("payload")]
        }
    }

    struct MockTransport;

    fn spawn_one_request_server(
        status_line: &str,
        response_body: &str,
    ) -> (String, JoinHandle<Value>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let status_line = status_line.to_owned();
        let response_body = response_body.to_owned();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("test server should accept one request");
            let mut reader = BufReader::new(stream.try_clone().expect("stream clone should work"));

            let mut request_line = String::new();
            reader
                .read_line(&mut request_line)
                .expect("request line should be readable");
            assert!(
                request_line.starts_with("POST / HTTP/1.1"),
                "unexpected request line: {request_line:?}"
            );

            let mut content_length = None;
            loop {
                let mut line = String::new();
                reader
                    .read_line(&mut line)
                    .expect("headers should be readable");
                if line == "\r\n" {
                    break;
                }
                let lower = line.to_ascii_lowercase();
                if let Some(value) = lower.strip_prefix("content-length:") {
                    content_length = Some(
                        value
                            .trim()
                            .parse::<usize>()
                            .expect("content-length should be numeric"),
                    );
                }
            }

            let content_length = content_length.expect("request should include content-length");
            let mut body = vec![0; content_length];
            reader
                .read_exact(&mut body)
                .expect("request body should be readable");
            let request_body = serde_json::from_slice::<Value>(&body)
                .expect("request body should be JSON-RPC JSON");

            let response = format!(
                "{status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{response_body}",
                response_body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("response should be writable");
            request_body
        });

        (endpoint, handle)
    }

    impl RpcTransport for MockTransport {
        fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
            assert_eq!(method, "ping");
            assert_eq!(params, vec![json!("payload")]);
            Ok(json!({ "ok": true }))
        }
    }

    #[test]
    fn typed_call_decodes_response() {
        let client = RpcClient::new(MockTransport);
        let response = client.call(PingParams).unwrap();
        assert_eq!(response, Pong { ok: true });
    }

    #[test]
    fn builds_graphene_websocket_call_envelope() {
        assert_eq!(
            build_graphene_ws_call_request(9, 2, "get_objects", vec![json!(["2.1.0"])]),
            json!({
                "id": 9,
                "method": "call",
                "params": [2, "get_objects", [["2.1.0"]]],
            })
        );
    }

    #[test]
    fn callback_params_prepend_local_callback_id() {
        assert_eq!(
            callback_params(77, vec![json!({ "trx": true })]),
            vec![json!(77), json!({ "trx": true })]
        );
    }

    #[test]
    fn parses_graphene_notice_shape() {
        let notice = parse_graphene_notice(&json!({
            "method": "notice",
            "params": [42, [{"id": "2.1.0"}]],
        }))
        .unwrap()
        .expect("notice should parse");

        assert_eq!(
            notice,
            GrapheneNotice {
                callback_id: 42,
                payload: json!([{ "id": "2.1.0" }]),
            }
        );
    }

    #[test]
    fn ignores_non_notice_messages_when_parsing_graphene_notice() {
        let notice = parse_graphene_notice(&json!({
            "id": 1,
            "method": "call",
            "result": true,
        }))
        .unwrap();

        assert!(notice.is_none());
    }

    #[test]
    fn rejects_malformed_graphene_notice() {
        let error = parse_graphene_notice(&json!({
            "method": "notice",
            "params": [],
        }))
        .unwrap_err();

        match error {
            RpcError::Protocol { method, message } => {
                assert_eq!(method, "notice");
                assert!(message.contains("callback id"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn builds_json_rpc_request_envelope() {
        assert_eq!(
            build_json_rpc_request(7, "about", vec![json!("x")]),
            json!({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "about",
                "params": ["x"],
            })
        );
    }

    #[test]
    fn http_transport_posts_json_rpc_request_and_decodes_success() {
        let (endpoint, handle) = spawn_one_request_server(
            "HTTP/1.1 200 OK",
            r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#,
        );
        let transport = HttpTransport::new(endpoint);

        let result = transport
            .call_raw("ping", vec![json!("payload")])
            .expect("HTTP transport should decode success response");

        assert_eq!(result, json!({ "ok": true }));
        assert_eq!(
            handle.join().expect("server thread should finish"),
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "ping",
                "params": ["payload"],
            })
        );
    }

    #[test]
    fn http_transport_surfaces_json_rpc_error_response() {
        let (endpoint, handle) = spawn_one_request_server(
            "HTTP/1.1 200 OK",
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"boom","data":{"detail":"x"}}}"#,
        );
        let transport = HttpTransport::new(endpoint);

        let error = transport.call_raw("explode", Vec::new()).unwrap_err();

        handle.join().expect("server thread should finish");
        match error {
            RpcError::JsonRpc {
                method,
                code,
                message,
                data,
            } => {
                assert_eq!(method, "explode");
                assert_eq!(code, -32000);
                assert_eq!(message, "boom");
                assert_eq!(data, Some(json!({ "detail": "x" })));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn http_transport_reports_malformed_json_body() {
        let (endpoint, handle) =
            spawn_one_request_server("HTTP/1.1 200 OK", r#"{"jsonrpc":"2.0","result":"#);
        let transport = HttpTransport::new(endpoint);

        let error = transport.call_raw("broken", Vec::new()).unwrap_err();

        handle.join().expect("server thread should finish");
        match error {
            RpcError::Http { method, message } => {
                assert_eq!(method, "broken");
                assert!(
                    message.contains("EOF") || message.contains("expected"),
                    "unexpected malformed JSON error: {message}"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn http_transport_reports_non_success_http_status() {
        let (endpoint, handle) = spawn_one_request_server(
            "HTTP/1.1 500 Internal Server Error",
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"server exploded"}}"#,
        );
        let transport = HttpTransport::new(endpoint);

        let error = transport.call_raw("server_error", Vec::new()).unwrap_err();

        handle.join().expect("server thread should finish");
        match error {
            RpcError::Http { method, message } => {
                assert_eq!(method, "server_error");
                assert!(
                    message.contains("500") && message.contains("server exploded"),
                    "unexpected HTTP status error: {message}"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parses_json_rpc_success_response() {
        let result = parse_json_rpc_response(
            "about",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": { "name": "swaplock" },
            }),
        )
        .unwrap();

        assert_eq!(result, json!({ "name": "swaplock" }));
    }

    #[test]
    fn parses_json_rpc_error_response() {
        let error = parse_json_rpc_response(
            "about",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {
                    "code": -32601,
                    "message": "method not found",
                    "data": { "method": "about" },
                },
            }),
        )
        .unwrap_err();

        match error {
            RpcError::JsonRpc {
                method,
                code,
                message,
                data,
            } => {
                assert_eq!(method, "about");
                assert_eq!(code, -32601);
                assert_eq!(message, "method not found");
                assert_eq!(data, Some(json!({ "method": "about" })));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn graphene_time_point_sec_deserializes_graphene_wire_format() {
        let value: GrapheneTimePointSec = serde_json::from_value(json!("2026-05-13T14:44:57"))
            .expect("Graphene timestamp should parse without timezone suffix");

        let expected = NaiveDate::from_ymd_opt(2026, 5, 13)
            .unwrap()
            .and_hms_opt(14, 44, 57)
            .unwrap();
        assert_eq!(value.naive_utc(), expected);
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            json!("2026-05-13T14:44:57")
        );
    }

    #[test]
    fn graphene_uint64_deserializes_number_and_decimal_string() {
        let from_number: GrapheneUInt64 = serde_json::from_value(json!(50_000_000_000_u64))
            .expect("Graphene uint64 should parse from JSON number");
        let from_string: GrapheneUInt64 = serde_json::from_value(json!("50000000000"))
            .expect("Graphene uint64 should parse from decimal string");

        assert_eq!(from_number.as_u64(), 50_000_000_000);
        assert_eq!(from_string.as_u64(), 50_000_000_000);
        assert_eq!(
            serde_json::to_value(from_string).unwrap(),
            json!(50_000_000_000_u64)
        );
    }

    #[test]
    fn graphene_int64_deserializes_number_and_decimal_string() {
        let from_number: GrapheneInt64 = serde_json::from_value(json!(-50_000_000_000_i64))
            .expect("Graphene int64 should parse from JSON number");
        let from_string: GrapheneInt64 = serde_json::from_value(json!("1000000000000000"))
            .expect("Graphene int64 should parse from decimal string");

        assert_eq!(from_number.as_i64(), -50_000_000_000);
        assert_eq!(from_string.as_i64(), 1_000_000_000_000_000);
        assert_eq!(
            serde_json::to_value(from_string).unwrap(),
            json!(1_000_000_000_000_000_i64)
        );
    }

    #[test]
    fn rejects_malformed_json_rpc_response() {
        let error =
            parse_json_rpc_response("about", json!({ "jsonrpc": "2.0", "id": 1 })).unwrap_err();

        match error {
            RpcError::Protocol { method, message } => {
                assert_eq!(method, "about");
                assert!(message.contains("missing result"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
