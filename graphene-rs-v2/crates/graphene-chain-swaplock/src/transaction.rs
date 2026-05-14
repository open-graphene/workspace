use std::fmt;

use graphene_codec::{to_graphene_bytes, EncodeError};
use graphene_rpc::{GrapheneInt64, GrapheneTimePointSec, RpcClient, RpcError, RpcTransport};
use graphene_signing::{signing_digest, ChainId, SignError, Signature, Signer};

use crate::broadcast::{BroadcastTransactionParams, BroadcastTransactionSynchronousParams};
use crate::generated::{
    Asset, AssetAssetId, DynamicGlobalPropertyObject, ExtensionsType, GetRequiredFeesParams,
    Operation, ProcessedTransaction, RequiredFee, SignedTransaction, Transaction,
    TransferOperation, TransferOperationFrom, TransferOperationTo, ValidateTransactionParams,
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

#[derive(Clone, Debug)]
pub struct TransferDraft {
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub asset_id: String,
}

pub fn build_transfer_operation(
    draft: TransferDraft,
) -> Result<TransferOperation, BuildTransactionError> {
    Ok(TransferOperation {
        fee: Asset {
            amount: GrapheneInt64::new(0),
            asset_id: parse_asset_id(&draft.asset_id)?,
        },
        from: TransferOperationFrom::try_from(draft.from.as_str()).map_err(|source| {
            BuildTransactionError::InvalidObjectId {
                field: "from",
                value: draft.from.clone(),
                source: source.to_string(),
            }
        })?,
        to: TransferOperationTo::try_from(draft.to.as_str()).map_err(|source| {
            BuildTransactionError::InvalidObjectId {
                field: "to",
                value: draft.to.clone(),
                source: source.to_string(),
            }
        })?,
        amount: Asset {
            amount: GrapheneInt64::new(draft.amount),
            asset_id: parse_asset_id(&draft.asset_id)?,
        },
        memo: None,
        extensions: ExtensionsType(vec![]),
    })
}

pub fn apply_required_fee(
    operation: &mut TransferOperation,
    fee: RequiredFee,
) -> Result<(), BuildTransactionError> {
    match fee {
        RequiredFee::Asset(asset) => {
            operation.fee = asset;
            Ok(())
        }
        other => Err(BuildTransactionError::UnsupportedFeeShape(format!(
            "{other:#?}"
        ))),
    }
}

pub fn fetch_required_fee_for_transfer<T>(
    client: &RpcClient<T>,
    operation: &TransferOperation,
    fee_asset_symbol_or_id: impl Into<String>,
) -> Result<Asset, BuildTransactionError>
where
    T: RpcTransport,
{
    let fees = client.call(GetRequiredFeesParams {
        ops: vec![Operation::Transfer(operation.clone())],
        asset_symbol_or_id: fee_asset_symbol_or_id.into(),
    })?;
    match fees.into_iter().next() {
        Some(RequiredFee::Asset(asset)) => Ok(asset),
        Some(other) => Err(BuildTransactionError::UnsupportedFeeShape(format!(
            "{other:#?}"
        ))),
        None => Err(BuildTransactionError::MissingRequiredFee),
    }
}

pub fn prepare_transaction(
    dynamic: &DynamicGlobalPropertyObject,
    operations: Vec<Operation>,
    expiration: GrapheneTimePointSec,
) -> Result<PreparedTransaction, BuildTransactionError> {
    Ok(PreparedTransaction::new(Transaction {
        ref_block_num: (dynamic.head_block_number & 0xffff) as u16,
        ref_block_prefix: ref_block_prefix(&dynamic.head_block_id)?,
        expiration,
        operations,
        extensions: ExtensionsType(vec![]),
    }))
}

fn parse_asset_id(value: &str) -> Result<AssetAssetId, BuildTransactionError> {
    AssetAssetId::try_from(value).map_err(|source| BuildTransactionError::InvalidObjectId {
        field: "asset_id",
        value: value.to_owned(),
        source: source.to_string(),
    })
}

fn ref_block_prefix(block_id: &str) -> Result<u32, BuildTransactionError> {
    let bytes = hex_to_bytes(block_id)?;
    if bytes.len() < 8 {
        return Err(BuildTransactionError::InvalidBlockId {
            value: block_id.to_owned(),
            reason: format!("expected at least 8 bytes, got {}", bytes.len()),
        });
    }
    Ok(u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
}

fn hex_to_bytes(value: &str) -> Result<Vec<u8>, BuildTransactionError> {
    if value.len() % 2 != 0 {
        return Err(BuildTransactionError::InvalidBlockId {
            value: value.to_owned(),
            reason: "hex string has odd length".to_owned(),
        });
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| Ok((hex_nibble(chunk[0], value)? << 4) | hex_nibble(chunk[1], value)?))
        .collect()
}

fn hex_nibble(byte: u8, value: &str) -> Result<u8, BuildTransactionError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(BuildTransactionError::InvalidBlockId {
            value: value.to_owned(),
            reason: format!("invalid hex byte 0x{byte:02x}"),
        }),
    }
}

#[derive(Debug)]
pub enum BuildTransactionError {
    InvalidObjectId {
        field: &'static str,
        value: String,
        source: String,
    },
    InvalidBlockId {
        value: String,
        reason: String,
    },
    MissingRequiredFee,
    UnsupportedFeeShape(String),
    Rpc(RpcError),
}

impl fmt::Display for BuildTransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObjectId {
                field,
                value,
                source,
            } => {
                write!(formatter, "invalid {field} object id {value:?}: {source}")
            }
            Self::InvalidBlockId { value, reason } => {
                write!(formatter, "invalid block id {value:?}: {reason}")
            }
            Self::MissingRequiredFee => write!(formatter, "required fee response was empty"),
            Self::UnsupportedFeeShape(shape) => {
                write!(formatter, "unsupported required fee shape: {shape}")
            }
            Self::Rpc(error) => write!(formatter, "RPC error while building transaction: {error}"),
        }
    }
}

impl std::error::Error for BuildTransactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rpc(error) => Some(error),
            Self::InvalidObjectId { .. }
            | Self::InvalidBlockId { .. }
            | Self::MissingRequiredFee
            | Self::UnsupportedFeeShape(_) => None,
        }
    }
}

impl From<RpcError> for BuildTransactionError {
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
