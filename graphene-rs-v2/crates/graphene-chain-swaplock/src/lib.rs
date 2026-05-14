//! Chain-local Rust types for Swaplock.
//!
//! This crate is generated from the Swaplock OpenRPC specification. The current
//! checked-in generated modules are an early v2 spike: schema structs come from
//! `typify`, while Graphene static variants use custom serde to preserve the
//! `[index, payload]` wire shape.

#![allow(clippy::all)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::enum_variant_names)]

pub mod generated {
    include!("generated/types.rs");
    include!("generated/variants.rs");
    include!("generated/rpc.rs");
}

pub mod broadcast {
    use super::generated::*;
    include!("generated/broadcast_rpc.rs");
}

mod codec;
mod transaction;

pub use generated::*;
pub use transaction::{
    apply_required_fee, broadcast_signed_transaction, broadcast_signed_transaction_synchronous,
    broadcast_signed_transaction_synchronous_typed, build_transfer_operation,
    fetch_required_fee_for_transfer, prepare_transaction, sign_transaction,
    validate_signed_transaction, BroadcastResultError, BuildTransactionError, PreparedTransaction,
    SignTransactionError, SignedTransactionEnvelope, SynchronousBroadcastResult, TransferDraft,
};
