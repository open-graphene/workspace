//! Chain-local Rust types for Acta.
//!
//! This crate is generated from the Acta OpenRPC specification. Schema structs
//! come from `typify`, while Graphene static variants use custom serde to
//! preserve the `[index, payload]` wire shape.

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
mod callbacks;
mod codec;
mod testnet;
mod transaction;
mod transfer;

pub use account::{lookup_exact_account_id, LookupAccountError};
pub use callbacks::{
    set_block_applied_callback, BlockAppliedNotice, BlockAppliedNoticeError,
    SetBlockAppliedCallbackParams,
};
pub use generated::*;
pub use graphene_transaction::broadcast::{BroadcastResultError, SynchronousBroadcastResult};
pub use testnet::{
    ACTA_TESTNET_FROM_ACCOUNT, ACTA_TESTNET_HTTP_URL, ACTA_TESTNET_TO_ACCOUNT,
    ACTA_TESTNET_TRANSFER_AMOUNT, ACTA_TESTNET_TRANSFER_ASSET, ACTA_TESTNET_WS_URL,
    PUBLIC_ACTA_TESTNET_WIF,
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
    prepare_transaction, BuildTransactionError, TransferDraft,
};
