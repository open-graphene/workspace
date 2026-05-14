use std::fmt;

use graphene_rpc::RpcError;

use crate::block_id::BlockIdError;

/// Chain-independent transfer input before it is converted into a generated operation.
///
/// The fields are intentionally low-level Graphene values, not wallet concepts:
/// callers provide already-resolved account object ids, an integer amount in the
/// asset's precision, and an asset object id such as `"1.3.0"`. Chain crates turn
/// this draft into their own generated `transfer_operation` type.
#[derive(Clone, Debug)]
pub struct TransferDraft {
    /// Sender account object id, for example `"1.2.100"`.
    pub from: String,
    /// Recipient account object id, for example `"1.2.101"`.
    pub to: String,
    /// Amount in the asset's smallest unit, matching Graphene's `share_type`.
    pub amount: i64,
    /// Asset object id for the amount being transferred, for example `"1.3.0"`.
    pub asset_id: String,
}

#[derive(Debug)]
pub enum BuildTransactionError {
    InvalidObjectId {
        field: &'static str,
        value: String,
        source: String,
    },
    InvalidBlockId(BlockIdError),
    MissingRequiredFee,
    UnsupportedFeeShape(String),
    Rpc(RpcError),
}

pub fn parse_object_id<T>(field: &'static str, value: &str) -> Result<T, BuildTransactionError>
where
    for<'a> T: TryFrom<&'a str>,
    for<'a> <T as TryFrom<&'a str>>::Error: fmt::Display,
{
    T::try_from(value).map_err(|source| BuildTransactionError::InvalidObjectId {
        field,
        value: value.to_owned(),
        source: source.to_string(),
    })
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
            Self::InvalidBlockId(error) => {
                write!(formatter, "invalid transaction head block id: {error}")
            }
            Self::Rpc(error) => {
                write!(formatter, "RPC error while building transaction: {error}")
            }
            Self::MissingRequiredFee => write!(formatter, "required fee response was empty"),
            Self::UnsupportedFeeShape(shape) => {
                write!(
                    formatter,
                    "unsupported required fee response shape: {shape}"
                )
            }
        }
    }
}

impl std::error::Error for BuildTransactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidBlockId(error) => Some(error),
            Self::Rpc(error) => Some(error),
            Self::InvalidObjectId { .. }
            | Self::MissingRequiredFee
            | Self::UnsupportedFeeShape(_) => None,
        }
    }
}

impl From<BlockIdError> for BuildTransactionError {
    fn from(error: BlockIdError) -> Self {
        Self::InvalidBlockId(error)
    }
}

impl From<RpcError> for BuildTransactionError {
    fn from(error: RpcError) -> Self {
        Self::Rpc(error)
    }
}
