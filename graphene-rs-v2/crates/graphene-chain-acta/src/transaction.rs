use std::fmt;

use graphene_codec::{to_graphene_bytes, EncodeError};
use graphene_rpc::{RpcClient, RpcError, RpcTransport};
use graphene_signing::{signing_digest, ChainId, SignError, Signature, Signer};

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

#[derive(Clone, Debug, PartialEq)]
pub struct SynchronousBroadcastResult {
    pub id: String,
    pub block_num: u32,
    pub trx_num: u32,
    pub raw: serde_json::Value,
}

impl TryFrom<serde_json::Value> for SynchronousBroadcastResult {
    type Error = BroadcastResultError;

    fn try_from(raw: serde_json::Value) -> Result<Self, Self::Error> {
        let id = raw
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or(BroadcastResultError::MissingField("id"))?
            .to_owned();
        let block_num = u64_to_u32_field(
            raw.get("block_num")
                .and_then(serde_json::Value::as_u64)
                .ok_or(BroadcastResultError::MissingField("block_num"))?,
            "block_num",
        )?;
        let trx_num = u64_to_u32_field(
            raw.get("trx_num")
                .and_then(serde_json::Value::as_u64)
                .ok_or(BroadcastResultError::MissingField("trx_num"))?,
            "trx_num",
        )?;

        Ok(Self {
            id,
            block_num,
            trx_num,
            raw,
        })
    }
}

fn u64_to_u32_field(value: u64, field: &'static str) -> Result<u32, BroadcastResultError> {
    u32::try_from(value).map_err(|_| BroadcastResultError::FieldOutOfRange { field, value })
}

#[derive(Debug)]
pub enum BroadcastResultError {
    Rpc(RpcError),
    MissingField(&'static str),
    FieldOutOfRange { field: &'static str, value: u64 },
}

impl fmt::Display for BroadcastResultError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rpc(error) => write!(
                formatter,
                "RPC error while broadcasting transaction: {error}"
            ),
            Self::MissingField(field) => {
                write!(
                    formatter,
                    "synchronous broadcast result is missing field {field}"
                )
            }
            Self::FieldOutOfRange { field, value } => write!(
                formatter,
                "synchronous broadcast result field {field} is out of u32 range: {value}"
            ),
        }
    }
}

impl std::error::Error for BroadcastResultError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rpc(error) => Some(error),
            Self::MissingField(_) | Self::FieldOutOfRange { .. } => None,
        }
    }
}

impl From<RpcError> for BroadcastResultError {
    fn from(error: RpcError) -> Self {
        Self::Rpc(error)
    }
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
