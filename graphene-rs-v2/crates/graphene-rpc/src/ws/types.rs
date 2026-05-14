use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, SyncSender};

use serde_json::Value;

use crate::RpcError;

use super::dispatcher::DispatcherCommand;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiHandle {
    pub(super) api_id: u64,
}

impl ApiHandle {
    pub fn id(&self) -> u64 {
        self.api_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackHandle {
    pub(super) callback_id: u64,
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
