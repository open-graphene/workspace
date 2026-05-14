//! Blocking Graphene WebSocket runtime.
//!
//! The public surface is the session-oriented API in [`GrapheneWebSocketSession`]
//! plus handle types used to keep Graphene API ids and callback ids scoped to a
//! single socket. Dispatcher and protocol modules are internal implementation
//! details; `GrapheneWebSocketOneShotTransport` is available for simple
//! one-call-per-socket flows.

mod dispatcher;
mod one_shot_transport;
mod protocol;
mod session;
mod types;

pub use one_shot_transport::GrapheneWebSocketOneShotTransport;
pub use session::{GrapheneWebSocketApiTransport, GrapheneWebSocketSession};
pub use types::{ApiHandle, CallbackSubscription};

#[cfg(test)]
// Re-export protocol helpers only for crate-level unit tests in lib.rs.
pub(crate) use protocol::{build_graphene_ws_call_request, callback_params, parse_graphene_notice};
