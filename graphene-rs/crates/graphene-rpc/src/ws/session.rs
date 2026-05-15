use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, SyncSender};
use std::sync::Arc;
use std::thread;

use serde_json::{json, Value};
use tungstenite::connect;

use crate::{OpenRpcCallbackParams, OpenRpcParams, RpcError, RpcTransport};

use super::dispatcher::{
    run_websocket_dispatcher, set_websocket_read_timeout, DispatcherCommand,
    DEFAULT_WS_COMMAND_QUEUE_CAPACITY, DEFAULT_WS_READ_TIMEOUT,
};
use super::protocol::{build_graphene_ws_call_request, callback_params};
use super::types::{ApiHandle, CallbackSubscription};

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

/// Transport adapter that lets generated `OpenRpcParams` calls use one API on
/// an existing `GrapheneWebSocketSession`.
pub struct GrapheneWebSocketApiTransport {
    session: Arc<GrapheneWebSocketSession>,
    api: ApiHandle,
}

impl RpcTransport for GrapheneWebSocketApiTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        self.session.call_api_raw(self.api.id(), method, params)
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
        Ok(ApiHandle::new(api_id))
    }

    pub fn call<P>(&self, api: &ApiHandle, params: P) -> Result<P::Response, RpcError>
    where
        P: OpenRpcParams,
        P::Response: serde::de::DeserializeOwned,
    {
        let raw = self.call_api_raw(api.id(), P::METHOD, params.into_positional_params())?;
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
            api.id(),
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
            CallbackSubscription::new(callback_id, self.commands.clone()),
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
