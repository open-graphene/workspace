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

mod account;
mod codec;
mod testnet;
mod transaction;
mod transfer;

pub use account::{lookup_exact_account_id, LookupAccountError};
pub use generated::*;
pub use graphene_transaction::broadcast::{BroadcastResultError, SynchronousBroadcastResult};
pub use testnet::{
    PUBLIC_SWAPLOCK_TESTNET_WIF, SWAPLOCK_TESTNET_FROM_ACCOUNT, SWAPLOCK_TESTNET_HTTP_URL,
    SWAPLOCK_TESTNET_TO_ACCOUNT, SWAPLOCK_TESTNET_WS_URL,
};
pub use transaction::{
    broadcast_signed_transaction, broadcast_signed_transaction_synchronous,
    broadcast_signed_transaction_synchronous_typed, broadcast_signed_transaction_with_callback,
    broadcast_signed_transaction_with_callback_typed, sign_transaction,
    validate_signed_transaction, PreparedTransaction, SignTransactionError,
    SignedTransactionEnvelope,
};
pub use transfer::{
    apply_required_fee, build_transfer_operation, fetch_required_fee_for_transfer,
    prepare_transaction, BuildTransactionError, TransferDraft,
};
