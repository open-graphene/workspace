use graphene_rpc::{
    ApiHandle, GrapheneWebSocketSession, OpenRpcCallbackParams, RpcClient, RpcError, RpcTransport,
};
use graphene_signing::{ChainId, Signature, Signer};
use graphene_transaction::broadcast::{BroadcastResultError, SynchronousBroadcastResult};
use graphene_transaction::signing::sign_transaction_bytes;

pub use graphene_transaction::signing::SignTransactionError;

use crate::broadcast::{BroadcastTransactionParams, BroadcastTransactionSynchronousParams};
use crate::generated::{
    ProcessedTransaction, SignedTransaction, Transaction, ValidateTransactionParams,
};

#[derive(Clone, Debug)]
pub struct PreparedTransaction {
    pub transaction: Transaction,
}

impl PreparedTransaction {
    pub fn new(transaction: Transaction) -> Self {
        Self { transaction }
    }

    pub fn into_transaction(self) -> Transaction {
        self.transaction
    }

    pub fn sign<S: Signer>(
        self,
        chain_id: &ChainId,
        signer: &S,
    ) -> Result<SignedTransactionEnvelope, SignTransactionError> {
        sign_transaction(chain_id, self.transaction, signer)
    }
}

#[derive(Clone, Debug)]
pub struct SignedTransactionEnvelope {
    pub transaction: Transaction,
    pub signatures: Vec<Signature>,
}

impl SignedTransactionEnvelope {
    pub fn into_generated(self) -> SignedTransaction {
        self.into()
    }
}

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

pub fn sign_transaction<S: Signer>(
    chain_id: &ChainId,
    transaction: Transaction,
    signer: &S,
) -> Result<SignedTransactionEnvelope, SignTransactionError> {
    let signatures = sign_transaction_bytes(chain_id, &transaction, signer)?;
    Ok(SignedTransactionEnvelope {
        transaction,
        signatures,
    })
}

pub fn validate_signed_transaction<T>(
    client: &RpcClient<T>,
    signed: SignedTransactionEnvelope,
) -> Result<ProcessedTransaction, RpcError>
where
    T: RpcTransport,
{
    client.call(ValidateTransactionParams {
        trx: signed.into_generated(),
    })
}

pub fn broadcast_signed_transaction<T>(
    client: &RpcClient<T>,
    signed: SignedTransactionEnvelope,
) -> Result<(), RpcError>
where
    T: RpcTransport,
{
    client.call(BroadcastTransactionParams {
        trx: signed.into_generated(),
    })
}

pub fn broadcast_signed_transaction_synchronous<T>(
    client: &RpcClient<T>,
    signed: SignedTransactionEnvelope,
) -> Result<serde_json::Value, RpcError>
where
    T: RpcTransport,
{
    client.call(BroadcastTransactionSynchronousParams {
        trx: signed.into_generated(),
    })
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

#[derive(Clone, Debug)]
pub struct BroadcastTransactionWithCallbackParams {
    pub trx: SignedTransaction,
}

impl OpenRpcCallbackParams for BroadcastTransactionWithCallbackParams {
    const METHOD: &'static str = "broadcast_transaction_with_callback";
    type Response = ();
    type Callback = serde_json::Value;

    fn into_positional_params_after_callback(self) -> Vec<serde_json::Value> {
        vec![serde_json::json!(self.trx)]
    }

    fn decode_response(value: serde_json::Value) -> Result<Self::Response, RpcError> {
        if value.is_null() {
            Ok(())
        } else {
            Err(RpcError::protocol(
                Self::METHOD,
                format!("expected null acknowledgement, got {value}"),
            ))
        }
    }

    fn decode_callback(value: serde_json::Value) -> Result<Self::Callback, RpcError> {
        Ok(value)
    }
}

pub fn broadcast_signed_transaction_with_callback(
    session: &GrapheneWebSocketSession,
    broadcast_api: &ApiHandle,
    signed: SignedTransactionEnvelope,
) -> Result<serde_json::Value, RpcError> {
    session.call_with_callback_once(
        broadcast_api,
        BroadcastTransactionWithCallbackParams {
            trx: signed.into_generated(),
        },
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
