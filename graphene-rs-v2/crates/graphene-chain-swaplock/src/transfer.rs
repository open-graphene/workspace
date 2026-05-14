use std::fmt;

use graphene_rpc::{GrapheneInt64, GrapheneTimePointSec, RpcClient, RpcError, RpcTransport};
use graphene_transaction::block_id::BlockIdError;
use graphene_transaction::transaction::compute_transaction_header_fields;

use crate::generated::{
    Asset, AssetAssetId, DynamicGlobalPropertyObject, ExtensionsType, GetRequiredFeesParams,
    Operation, RequiredFee, Transaction, TransferOperation, TransferOperationFrom,
    TransferOperationTo,
};
use crate::transaction::PreparedTransaction;

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
    let asset_id = parse_asset_id(&draft.asset_id)?;

    Ok(TransferOperation {
        fee: Asset {
            amount: GrapheneInt64::new(0),
            asset_id: asset_id.clone(),
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
            asset_id,
        },
        memo: None,
        extensions: ExtensionsType(vec![]),
    })
}

pub fn apply_required_fee(
    operation: &mut TransferOperation,
    fee: RequiredFee,
) -> Result<(), BuildTransactionError> {
    operation.fee = required_fee_asset(fee)?;
    Ok(())
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
    let Some(fee) = fees.into_iter().next() else {
        return Err(BuildTransactionError::MissingRequiredFee);
    };
    required_fee_asset(fee)
}

fn required_fee_asset(fee: RequiredFee) -> Result<Asset, BuildTransactionError> {
    match fee {
        RequiredFee::Asset(asset) => Ok(asset),
        other => Err(BuildTransactionError::UnsupportedFeeShape(format!(
            "{other:#?}"
        ))),
    }
}

pub fn prepare_transaction(
    dynamic: &DynamicGlobalPropertyObject,
    operations: Vec<Operation>,
    expiration: GrapheneTimePointSec,
) -> Result<PreparedTransaction, BuildTransactionError> {
    let header = compute_transaction_header_fields(
        &dynamic.head_block_id,
        dynamic.head_block_number,
        expiration,
    )?;

    Ok(PreparedTransaction::new(Transaction {
        ref_block_num: header.ref_block_num,
        ref_block_prefix: header.ref_block_prefix,
        expiration: header.expiration,
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

impl From<BlockIdError> for BuildTransactionError {
    fn from(error: BlockIdError) -> Self {
        Self::InvalidBlockId {
            value: error.value,
            reason: error.reason,
        }
    }
}

impl From<RpcError> for BuildTransactionError {
    fn from(error: RpcError) -> Self {
        Self::Rpc(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::{
        DynamicGlobalPropertyObjectCurrentWitness, DynamicGlobalPropertyObjectRecentSlotsFilled,
    };

    fn transfer_draft() -> TransferDraft {
        TransferDraft {
            from: "1.2.100".to_owned(),
            to: "1.2.101".to_owned(),
            amount: 123,
            asset_id: "1.3.0".to_owned(),
        }
    }

    fn dynamic_with_head_block_id(head_block_id: &str) -> DynamicGlobalPropertyObject {
        DynamicGlobalPropertyObject {
            accounts_registered_this_interval: 0,
            current_aslot: 0u64.into(),
            current_witness: DynamicGlobalPropertyObjectCurrentWitness::try_from("1.6.1")
                .expect("valid witness id"),
            dynamic_flags: 0,
            head_block_id: head_block_id.to_owned(),
            head_block_number: 0x12345,
            last_budget_time: "2026-05-13T20:00:00".parse().expect("valid timestamp"),
            last_irreversible_block_num: 0,
            last_vote_tally_time: "2026-05-13T20:00:00".parse().expect("valid timestamp"),
            next_maintenance_time: "2026-05-13T20:00:00".parse().expect("valid timestamp"),
            recent_slots_filled: DynamicGlobalPropertyObjectRecentSlotsFilled::try_from("0")
                .expect("valid u128 decimal"),
            recently_missed_count: 0,
            time: "2026-05-13T20:00:00".parse().expect("valid timestamp"),
            total_inactive: GrapheneInt64::new(0),
            total_pob: GrapheneInt64::new(0),
            witness_budget: GrapheneInt64::new(0),
        }
    }

    #[test]
    fn transfer_draft_builds_transfer_operation_with_zero_fee_placeholder() {
        let transfer = build_transfer_operation(transfer_draft()).expect("draft should build");

        assert_eq!(transfer.from.as_str(), "1.2.100");
        assert_eq!(transfer.to.as_str(), "1.2.101");
        assert_eq!(transfer.amount.amount.as_i64(), 123);
        assert_eq!(transfer.amount.asset_id.as_str(), "1.3.0");
        assert_eq!(transfer.fee.amount.as_i64(), 0);
        assert_eq!(transfer.fee.asset_id.as_str(), "1.3.0");
        assert!(transfer.memo.is_none());
    }

    #[test]
    fn transfer_draft_rejects_invalid_from_id() {
        let mut draft = transfer_draft();
        draft.from = "not-an-id".to_owned();

        let error = build_transfer_operation(draft).expect_err("bad from id should fail");

        match error {
            BuildTransactionError::InvalidObjectId { field, value, .. } => {
                assert_eq!(field, "from");
                assert_eq!(value, "not-an-id");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn apply_required_fee_rejects_nested_proposal_fee_shape() {
        let mut transfer = build_transfer_operation(transfer_draft()).expect("draft should build");
        let fee = RequiredFee::ProposalCreate((
            Asset {
                amount: GrapheneInt64::new(1),
                asset_id: AssetAssetId::try_from("1.3.0").expect("valid asset id"),
            },
            vec![],
        ));

        let error = apply_required_fee(&mut transfer, fee).expect_err("nested fee should fail");

        assert!(matches!(
            error,
            BuildTransactionError::UnsupportedFeeShape(_)
        ));
    }

    #[test]
    fn prepare_transaction_derives_ref_block_fields_from_dynamic_global_properties() {
        let dynamic = dynamic_with_head_block_id(
            "00012345a1b2c3d4ffffffffffffffffffffffffffffffffffffffffffffffff",
        );
        let transfer = build_transfer_operation(transfer_draft()).expect("draft should build");
        let expiration = "2026-05-13T20:05:00".parse().expect("valid timestamp");

        let prepared =
            prepare_transaction(&dynamic, vec![Operation::Transfer(transfer)], expiration)
                .expect("transaction should prepare");
        let transaction = prepared.into_transaction();

        assert_eq!(transaction.ref_block_num, 0x2345);
        assert_eq!(transaction.ref_block_prefix, 0xd4c3b2a1);
        assert_eq!(transaction.expiration, expiration);
        assert_eq!(transaction.operations.len(), 1);
    }

    #[test]
    fn prepare_transaction_rejects_invalid_head_block_id_hex() {
        let dynamic = dynamic_with_head_block_id("not-hex");
        let expiration = "2026-05-13T20:05:00".parse().expect("valid timestamp");

        let error = prepare_transaction(&dynamic, vec![], expiration)
            .expect_err("bad head block id should fail");

        assert!(matches!(
            error,
            BuildTransactionError::InvalidBlockId { .. }
        ));
    }
}
