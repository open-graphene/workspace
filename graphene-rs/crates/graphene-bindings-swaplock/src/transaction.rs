use graphene_rpc::{ApiHandle, GrapheneWebSocketSession, RpcClient, RpcError, RpcTransport};
use graphene_signing::Signature;
use graphene_transaction::broadcast::{
    BroadcastResultError,
    BroadcastTransactionWithCallbackParams as SharedBroadcastTransactionWithCallbackParams,
    SynchronousBroadcastResult,
};
use graphene_transaction::signing::{
    PreparedTransaction as SharedPreparedTransaction,
    SignedTransactionEnvelope as SharedSignedTransactionEnvelope,
};

pub use graphene_transaction::signing::{sign_transaction, SignTransactionError};

use crate::broadcast::{BroadcastTransactionParams, BroadcastTransactionSynchronousParams};
use crate::generated::{
    ProcessedTransaction, SignedTransaction, Transaction, ValidateTransactionParams,
};

pub type PreparedTransaction = SharedPreparedTransaction<Transaction>;
pub type SignedTransactionEnvelope = SharedSignedTransactionEnvelope<Transaction>;
pub type BroadcastTransactionWithCallbackParams =
    SharedBroadcastTransactionWithCallbackParams<SignedTransaction>;

impl From<SignedTransactionEnvelope> for SignedTransaction {
    fn from(envelope: SignedTransactionEnvelope) -> Self {
        let Transaction {
            expiration,
            extensions,
            operations,
            ref_block_num,
            ref_block_prefix,
        } = envelope.transaction;

        Self {
            expiration,
            extensions,
            operations,
            ref_block_num,
            ref_block_prefix,
            signatures: envelope
                .signatures
                .into_iter()
                .map(Signature::to_hex)
                .collect(),
        }
    }
}

pub fn validate_signed_transaction<T>(
    client: &RpcClient<T>,
    signed: SignedTransactionEnvelope,
) -> Result<ProcessedTransaction, RpcError>
where
    T: RpcTransport,
{
    client.call(ValidateTransactionParams { trx: signed.into() })
}

pub fn broadcast_signed_transaction<T>(
    client: &RpcClient<T>,
    signed: SignedTransactionEnvelope,
) -> Result<(), RpcError>
where
    T: RpcTransport,
{
    client.call(BroadcastTransactionParams { trx: signed.into() })
}

pub fn broadcast_signed_transaction_synchronous<T>(
    client: &RpcClient<T>,
    signed: SignedTransactionEnvelope,
) -> Result<serde_json::Value, RpcError>
where
    T: RpcTransport,
{
    client.call(BroadcastTransactionSynchronousParams { trx: signed.into() })
}

pub fn broadcast_signed_transaction_synchronous_typed<T>(
    client: &RpcClient<T>,
    signed: SignedTransactionEnvelope,
) -> Result<SynchronousBroadcastResult, BroadcastResultError>
where
    T: RpcTransport,
{
    let raw = broadcast_signed_transaction_synchronous(client, signed)?;
    SynchronousBroadcastResult::try_from(raw)
}

pub fn broadcast_signed_transaction_with_callback(
    session: &GrapheneWebSocketSession,
    broadcast_api: &ApiHandle,
    signed: SignedTransactionEnvelope,
) -> Result<serde_json::Value, RpcError> {
    session.call_with_callback_once(
        broadcast_api,
        BroadcastTransactionWithCallbackParams { trx: signed.into() },
    )
}

pub fn broadcast_signed_transaction_with_callback_typed(
    session: &GrapheneWebSocketSession,
    broadcast_api: &ApiHandle,
    signed: SignedTransactionEnvelope,
) -> Result<SynchronousBroadcastResult, BroadcastResultError> {
    let raw = broadcast_signed_transaction_with_callback(session, broadcast_api, signed)?;
    SynchronousBroadcastResult::try_from(raw)
}
