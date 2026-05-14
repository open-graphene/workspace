use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

use serde_json::Value;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

use crate::jsonrpc::parse_json_rpc_response;
use crate::RpcError;

use super::protocol::{parse_graphene_notice, read_websocket_json};

pub(super) type NoticeCallback = Box<dyn FnMut(Value) + Send + 'static>;

pub(super) const DEFAULT_WS_COMMAND_QUEUE_CAPACITY: usize = 128;
pub(super) const DEFAULT_WS_READ_TIMEOUT: Duration = Duration::from_millis(50);

pub(super) enum DispatcherCommand {
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

pub(super) fn run_websocket_dispatcher(
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

pub(super) fn set_websocket_read_timeout(
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
