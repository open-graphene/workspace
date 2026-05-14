use std::fmt;

use graphene_codec::{to_graphene_bytes, EncodeError};
use graphene_rpc::{RpcClient, RpcError, RpcTransport};
use graphene_signing::{signing_digest, ChainId, SignError, Signature, Signer};
use graphene_transaction::broadcast::{BroadcastResultError, SynchronousBroadcastResult};

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
    let transaction_bytes = to_graphene_bytes(&transaction)?;
    let digest = signing_digest(chain_id, &transaction_bytes);
    let signature = signer.sign_digest(&digest)?;

    Ok(SignedTransactionEnvelope {
        transaction,
        signatures: vec![signature],
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

#[derive(Debug)]
pub enum SignTransactionError {
    Encode(EncodeError),
    Sign(SignError),
}

impl fmt::Display for SignTransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(error) => write!(formatter, "failed to encode transaction: {error}"),
            Self::Sign(error) => write!(formatter, "failed to sign transaction digest: {error}"),
        }
    }
}

impl std::error::Error for SignTransactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::Sign(error) => Some(error),
        }
    }
}

impl From<EncodeError> for SignTransactionError {
    fn from(error: EncodeError) -> Self {
        Self::Encode(error)
    }
}

impl From<SignError> for SignTransactionError {
    fn from(error: SignError) -> Self {
        Self::Sign(error)
    }
}
