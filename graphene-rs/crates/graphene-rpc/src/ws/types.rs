use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, SyncSender};

use crate::RpcError;

use super::dispatcher::DispatcherCommand;

/// Session-local Graphene API handle.
///
/// Graphene API ids are scoped to the WebSocket connection that returned them.
/// Do not reuse an `ApiHandle` with another `GrapheneWebSocketSession`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiHandle {
    api_id: u64,
}

impl ApiHandle {
    pub(super) fn new(api_id: u64) -> Self {
        Self { api_id }
    }

    pub fn id(&self) -> u64 {
        self.api_id
    }
}

/// RAII subscription for a callback route registered with the dispatcher.
///
/// Dropping the subscription removes only the local callback route. It does not
/// send a chain-specific unsubscribe RPC because Graphene APIs expose different
/// unsubscribe semantics per callback method.
pub struct CallbackSubscription {
    callback_id: u64,
    commands: SyncSender<DispatcherCommand>,
    closed: AtomicBool,
}

impl CallbackSubscription {
    pub(super) fn new(callback_id: u64, commands: SyncSender<DispatcherCommand>) -> Self {
        Self {
            callback_id,
            commands,
            closed: AtomicBool::new(false),
        }
    }

    pub fn id(&self) -> u64 {
        self.callback_id
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
