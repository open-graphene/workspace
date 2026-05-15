//! Rust data models and minimal runtime helpers for Swaplock.
//!
//! The generated modules provide schema structs and Graphene `static_variant`
//! enums. Hand-written modules may provide chain-local transaction, signing,
//! and broadcast helpers on top of the shared runtime crates.

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
    include!("generated/metadata.rs");
    include!("generated/rpc.rs");
}

pub mod broadcast {
    use super::generated::*;
    include!("generated/broadcast_rpc.rs");
}

mod account;
mod codec;
mod dynamic_global_properties;
mod scalar;
mod testnet;
mod transaction;
mod transfer;

pub use account::{lookup_exact_account_id, LookupAccountError};
pub use dynamic_global_properties::{
    subscribe_dynamic_global_properties, SetSubscribeCallbackParams,
};
pub use generated::*;
pub use graphene_rpc::database_callbacks::{
    set_block_applied_callback, BlockAppliedNotice, BlockAppliedNoticeError,
    SetBlockAppliedCallbackParams,
};
pub use graphene_transaction::broadcast::{BroadcastResultError, SynchronousBroadcastResult};
pub use testnet::{
    PUBLIC_SWAPLOCK_TESTNET_WIF, SWAPLOCK_TESTNET_FROM_ACCOUNT, SWAPLOCK_TESTNET_HTTP_URL,
    SWAPLOCK_TESTNET_TO_ACCOUNT, SWAPLOCK_TESTNET_TRANSFER_AMOUNT, SWAPLOCK_TESTNET_TRANSFER_ASSET,
    SWAPLOCK_TESTNET_WS_URL,
};
pub use transaction::{
    broadcast_signed_transaction, broadcast_signed_transaction_synchronous,
    broadcast_signed_transaction_synchronous_typed, broadcast_signed_transaction_with_callback,
    broadcast_signed_transaction_with_callback_typed, sign_transaction,
    validate_signed_transaction, BroadcastTransactionWithCallbackParams, PreparedTransaction,
    SignTransactionError, SignedTransactionEnvelope,
};
pub use transfer::{
    apply_required_fee, build_transfer_operation, fetch_required_fee_for_transfer,
    prepare_transaction, prepare_transfer_transaction, BuildTransactionError, TransferDraft,
};
