use graphene_chain_bitshares::{
    operations::Operation,
    transaction::OperationResult,
    types::{
        account, account_balance, account_history, account_statistics, asset, asset_bitasset_data,
        asset_dynamic_data, balance, base, blinded_balance, block_summary, budget_record, buyback,
        call_order, chain_property, collateral_bid, committee_member, credit_deal,
        credit_deal_summary, credit_offer, custom, custom_authority, dynamic_global_property,
        fba_accumulator, force_settlement, global_property, htlc, limit_order, liquidity_pool,
        null, operation_history, proposal, reserved0, samet_fund, special_authority, ticket,
        transaction_history, vesting_balance, withdraw_permission, witness, witness_schedule,
        worker,
    },
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn deserializes_all_captured_live_rpc_fixtures() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0usize;
    let mut skipped = Vec::new();

    for entry in fs::read_dir(&fixtures_dir).expect("fixtures directory should be readable") {
        let path = entry
            .expect("fixture directory entry should be readable")
            .path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }

        let envelope = read_fixture_envelope(&path);
        let family = fixture_family(&envelope, &path);
        let object = fixture_object(&envelope, &path, family);

        if object.is_null() {
            assert_null_fixture_has_explicit_reason(&envelope, &path, family);
            skipped.push(family.to_owned());
            continue;
        }

        deserialize_present_fixture_object(family, object);
        checked += 1;
    }

    assert!(
        checked > 0,
        "at least one live fixture should contain an object"
    );
    eprintln!(
        "deserialized {checked} fixture objects; skipped {} null/error fixtures: {}",
        skipped.len(),
        skipped.join(", ")
    );
}

fn read_fixture_envelope(path: &Path) -> Value {
    let fixture = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    serde_json::from_str(&fixture)
        .unwrap_or_else(|error| panic!("{} should be valid JSON: {error}", path.display()))
}

fn fixture_family<'a>(envelope: &'a Value, path: &Path) -> &'a str {
    envelope
        .get("family")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{} should contain string family", path.display()))
}

fn fixture_object<'a>(envelope: &'a Value, path: &Path, family: &str) -> &'a Value {
    envelope.get("object").unwrap_or_else(|| {
        panic!(
            "{family} fixture {} should contain object field",
            path.display()
        )
    })
}

fn assert_null_fixture_has_explicit_reason(envelope: &Value, path: &Path, family: &str) {
    let has_error = envelope.get("error").is_some_and(|error| !error.is_null());
    let has_skip_reason = envelope
        .get("skip_reason")
        .and_then(Value::as_str)
        .is_some_and(|skip_reason| !skip_reason.trim().is_empty());

    assert!(
        has_error || has_skip_reason,
        "{family} fixture {} has object:null without non-empty skip_reason or RPC error metadata",
        path.display()
    );
}

fn deserialize_present_fixture_object(family: &str, object: &Value) {
    match family {
        "account" => assert_deserializes::<account::Object>(family, object),
        "account_balance" => assert_deserializes::<account_balance::Object>(family, object),
        "account_history" => assert_deserializes::<account_history::Object>(family, object),
        "account_statistics" => assert_deserializes::<account_statistics::Object>(family, object),
        "asset" => assert_deserializes::<asset::Object>(family, object),
        "asset_bitasset_data" => assert_deserializes::<asset_bitasset_data::Object>(family, object),
        "asset_dynamic_data" => assert_deserializes::<asset_dynamic_data::Object>(family, object),
        "balance" => assert_deserializes::<balance::Object>(family, object),
        "base" => assert_deserializes::<base::Object>(family, object),
        "blinded_balance" => assert_deserializes::<blinded_balance::Object>(family, object),
        "block_summary" => assert_deserializes::<block_summary::Object>(family, object),
        "budget_record" => assert_deserializes::<budget_record::Object>(family, object),
        "buyback" => assert_deserializes::<buyback::Object>(family, object),
        "call_order" => assert_deserializes::<call_order::Object>(family, object),
        "chain_property" => assert_deserializes::<chain_property::Object>(family, object),
        "collateral_bid" => assert_deserializes::<collateral_bid::Object>(family, object),
        "committee_member" => assert_deserializes::<committee_member::Object>(family, object),
        "credit_deal" => assert_deserializes::<credit_deal::Object>(family, object),
        "credit_deal_summary" => assert_deserializes::<credit_deal_summary::Object>(family, object),
        "credit_offer" => assert_deserializes::<credit_offer::Object>(family, object),
        "custom" => assert_deserializes::<custom::Object>(family, object),
        "custom_authority" => assert_deserializes::<custom_authority::Object>(family, object),
        "dynamic_global_property" => {
            assert_deserializes::<dynamic_global_property::Object>(family, object)
        }
        "fba_accumulator" => assert_deserializes::<fba_accumulator::Object>(family, object),
        "force_settlement" => assert_deserializes::<force_settlement::Object>(family, object),
        "global_property" => assert_deserializes::<global_property::Object>(family, object),
        "htlc" => assert_deserializes::<htlc::Object>(family, object),
        "limit_order" => assert_deserializes::<limit_order::Object>(family, object),
        "liquidity_pool" => assert_deserializes::<liquidity_pool::Object>(family, object),
        "null" => assert_deserializes::<null::Object>(family, object),
        "operation_history" => assert_operation_history_fixture_uses_typed_balance_claim(object),
        "proposal" => assert_proposal_fixture_uses_typed_operations(object),
        "reserved0" => assert_deserializes::<reserved0::Object>(family, object),
        "samet_fund" => assert_deserializes::<samet_fund::Object>(family, object),
        "special_authority" => assert_deserializes::<special_authority::Object>(family, object),
        "ticket" => assert_deserializes::<ticket::Object>(family, object),
        "transaction_history" => assert_transaction_history_fixture_uses_typed_operations(object),
        "vesting_balance" => assert_deserializes::<vesting_balance::Object>(family, object),
        "withdraw_permission" => assert_deserializes::<withdraw_permission::Object>(family, object),
        "witness" => assert_deserializes::<witness::Object>(family, object),
        "witness_schedule" => assert_deserializes::<witness_schedule::Object>(family, object),
        "worker" => assert_deserializes::<worker::Object>(family, object),
        other => panic!("no fixture deserializer is registered for family {other}"),
    };
}

fn assert_deserializes<T>(family: &str, object: &Value)
where
    T: DeserializeOwned,
{
    let _: T = deserialize(family, object);
}

fn assert_operation_history_fixture_uses_typed_balance_claim(object: &Value) {
    let object: operation_history::Object = deserialize("operation_history", object);

    assert_typed_operation("operation_history", "op", &object.op);
    assert_typed_result("operation_history", "result", &object.result);
    assert_eq!(object.op.tag(), 37);
    assert_eq!(object.result.tag(), 0);
    match &object.op {
        Operation::BalanceClaim(operation) => {
            assert_eq!(operation.deposit_to_account.to_string(), "1.2.90744");
            assert_eq!(operation.balance_to_claim.to_string(), "1.15.10747");
            assert_eq!(operation.total_claimed.amount, 81891883);
            assert_eq!(operation.total_claimed.asset_id.to_string(), "1.3.0");
        }
        other => panic!(
            "operation_history fixture should deserialize tag 37 as BalanceClaim, got operation tag {}",
            other.tag()
        ),
    }
}

fn assert_proposal_fixture_uses_typed_operations(object: &Value) {
    let object: proposal::Object = deserialize("proposal", object);
    assert_operations_typed(
        "proposal",
        "proposed_transaction.operations",
        &object.proposed_transaction.operations,
    );
}

fn assert_transaction_history_fixture_uses_typed_operations(object: &Value) {
    let object: transaction_history::Object = deserialize("transaction_history", object);
    assert_operations_typed(
        "transaction_history",
        "trx.operations",
        &object.trx.transaction.operations,
    );
}

fn assert_operations_typed(family: &str, path: &str, operations: &[Operation]) {
    for (index, operation) in operations.iter().enumerate() {
        assert_typed_operation(family, &format!("{path}[{index}]"), operation);
    }
}

fn assert_typed_operation(family: &str, path: &str, operation: &Operation) {
    assert!(
        operation.is_typed(),
        "{family} fixture {path} contains unsupported operation tag {}",
        operation.tag()
    );
}

fn assert_typed_result(family: &str, path: &str, result: &OperationResult) {
    assert!(
        result.is_typed(),
        "{family} fixture {path} contains unsupported operation result tag {}",
        result.tag()
    );
}

fn deserialize<T>(family: &str, object: &Value) -> T
where
    T: DeserializeOwned,
{
    serde_json::from_value::<T>(object.clone())
        .unwrap_or_else(|error| panic!("{family} fixture should deserialize: {error}"))
}
