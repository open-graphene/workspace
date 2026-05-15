use graphene_rpc::{GrapheneInt64, GrapheneTimePointSec, RpcClient, RpcTransport};
use graphene_transaction::transaction::compute_transaction_header_fields;
pub use graphene_transaction::transfer::{parse_object_id, BuildTransactionError, TransferDraft};

use crate::generated::{
    Asset, AssetAssetId, DynamicGlobalPropertyObject, ExtensionsType,
    GetDynamicGlobalPropertiesParams, GetRequiredFeesParams, Operation, RequiredFee, Transaction,
    TransferOperation, TransferOperationFrom, TransferOperationTo,
};
use crate::transaction::PreparedTransaction;

/// Converts a chain-independent [`TransferDraft`] into this chain's generated transfer operation.
///
/// The operation is built with a zero fee placeholder. Call
/// [`fetch_required_fee_for_transfer`] or [`prepare_transfer_transaction`] before
/// signing or broadcasting.
pub fn build_transfer_operation(
    draft: TransferDraft,
) -> Result<TransferOperation, BuildTransactionError> {
    let asset_id: AssetAssetId = parse_object_id("asset_id", &draft.asset_id)?;

    Ok(TransferOperation {
        fee: Asset {
            amount: GrapheneInt64::new(0),
            asset_id: asset_id.clone(),
        },
        from: parse_object_id("from", &draft.from)?,
        to: parse_object_id("to", &draft.to)?,
        amount: Asset {
            amount: GrapheneInt64::new(draft.amount),
            asset_id,
        },
        memo: None,
        extensions: ExtensionsType(vec![]),
    })
}

/// Applies a decoded required-fee response to a transfer operation.
///
/// Graphene can return nested fee shapes for proposal-like operations. A plain
/// transfer must receive a single asset fee; nested shapes are rejected so callers
/// do not accidentally sign a transaction with an unsupported fee model.
pub fn apply_required_fee(
    operation: &mut TransferOperation,
    fee: RequiredFee,
) -> Result<(), BuildTransactionError> {
    operation.fee = required_fee_asset(fee)?;
    Ok(())
}

/// Fetches the required fee for a transfer operation from the database API.
///
/// `fee_asset_symbol_or_id` is passed through to `get_required_fees`, so callers
/// can choose the fee asset by symbol or object id according to what the node
/// accepts. The returned fee is a generated chain-local `asset` value ready to
/// assign to `transfer.fee`.
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

/// Prepares a signed-transfer-ready transaction using the proven low-level flow.
///
/// This is a thin convenience helper, not a wallet abstraction. It builds the
/// transfer operation, fetches the required fee, reads dynamic global properties,
/// derives the expiration from the current chain time plus `expiration_lifetime`,
/// and returns a [`PreparedTransaction`] for the caller to sign. The caller still
/// owns account lookup, chain-id lookup, signer selection, and broadcast choice.
pub fn prepare_transfer_transaction<T>(
    client: &RpcClient<T>,
    draft: TransferDraft,
    fee_asset_symbol_or_id: impl Into<String>,
    expiration_lifetime: chrono::Duration,
) -> Result<PreparedTransaction, BuildTransactionError>
where
    T: RpcTransport,
{
    let mut transfer = build_transfer_operation(draft)?;
    transfer.fee = fetch_required_fee_for_transfer(client, &transfer, fee_asset_symbol_or_id)?;
    let dynamic = client.call(GetDynamicGlobalPropertiesParams)?;
    let expiration = GrapheneTimePointSec::new(dynamic.time.naive_utc() + expiration_lifetime);
    prepare_transaction(&dynamic, vec![Operation::Transfer(transfer)], expiration)
}

/// Builds a prepared transaction from already-constructed operations and dynamic globals.
///
/// Use this when you need manual control over the operation list. It only fills
/// transaction header fields from `dynamic`; it does not fetch fees, resolve
/// accounts, sign, validate, or broadcast.
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

        assert!(matches!(error, BuildTransactionError::InvalidBlockId(_)));
    }

    struct TransferFlowTransport;

    impl RpcTransport for TransferFlowTransport {
        fn call_raw(
            &self,
            method: &str,
            params: Vec<serde_json::Value>,
        ) -> Result<serde_json::Value, graphene_rpc::RpcError> {
            match method {
                "get_required_fees" => {
                    assert_eq!(params.len(), 2);
                    assert_eq!(params[1], serde_json::json!("1.3.0"));
                    serde_json::to_value(vec![RequiredFee::Asset(Asset {
                        amount: GrapheneInt64::new(7),
                        asset_id: AssetAssetId::try_from("1.3.0").expect("valid asset id"),
                    })])
                    .map_err(|source| graphene_rpc::RpcError::decode("get_required_fees", source))
                }
                "get_dynamic_global_properties" => {
                    serde_json::to_value(dynamic_with_head_block_id(
                        "00012345a1b2c3d4ffffffffffffffffffffffffffffffffffffffffffffffff",
                    ))
                    .map_err(|source| {
                        graphene_rpc::RpcError::decode("get_dynamic_global_properties", source)
                    })
                }
                other => panic!("unexpected RPC method: {other}"),
            }
        }
    }

    #[test]
    fn prepare_transfer_transaction_builds_fee_and_header_from_rpc_flow() {
        let client = RpcClient::new(TransferFlowTransport);

        let prepared = prepare_transfer_transaction(
            &client,
            transfer_draft(),
            "1.3.0",
            chrono::Duration::minutes(5),
        )
        .expect("transfer flow should prepare transaction");
        let transaction = prepared.into_transaction();

        assert_eq!(transaction.ref_block_num, 0x2345);
        assert_eq!(transaction.ref_block_prefix, 0xd4c3b2a1);
        assert_eq!(
            transaction.expiration,
            "2026-05-13T20:05:00".parse().expect("valid timestamp")
        );
        let Operation::Transfer(transfer) = &transaction.operations[0] else {
            panic!("expected transfer operation");
        };
        assert_eq!(transfer.fee.amount.as_i64(), 7);
        assert_eq!(transfer.fee.asset_id.as_str(), "1.3.0");
    }
}
