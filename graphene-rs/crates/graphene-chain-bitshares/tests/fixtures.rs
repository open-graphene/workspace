use graphene_chain_bitshares::types::{
    account, account_balance, account_history, account_statistics, asset, asset_bitasset_data,
    asset_dynamic_data, balance, base, blinded_balance, block_summary, budget_record, buyback,
    call_order, chain_property, collateral_bid, committee_member, credit_deal, credit_deal_summary,
    credit_offer, custom, custom_authority, dynamic_global_property, fba_accumulator,
    force_settlement, global_property, htlc, limit_order, liquidity_pool, null, operation_history,
    proposal, reserved0, samet_fund, special_authority, ticket, transaction_history,
    vesting_balance, withdraw_permission, witness, witness_schedule, worker,
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

        let fixture = fs::read_to_string(&path).expect("fixture file should be readable");
        let envelope: Value = serde_json::from_str(&fixture)
            .unwrap_or_else(|error| panic!("{} should be valid JSON: {error}", path.display()));
        let family = envelope
            .get("family")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{} should contain string family", path.display()));
        let object = envelope
            .get("object")
            .cloned()
            .unwrap_or_else(|| panic!("{} should contain object field", path.display()));

        if object.is_null() {
            skipped.push(family.to_owned());
            continue;
        }

        deserialize_fixture_object(family, object);
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

fn deserialize_fixture_object(family: &str, object: Value) {
    match family {
        "account" => deserialize::<account::Object>(family, object),
        "account_balance" => deserialize::<account_balance::Object>(family, object),
        "account_history" => deserialize::<account_history::Object>(family, object),
        "account_statistics" => deserialize::<account_statistics::Object>(family, object),
        "asset" => deserialize::<asset::Object>(family, object),
        "asset_bitasset_data" => deserialize::<asset_bitasset_data::Object>(family, object),
        "asset_dynamic_data" => deserialize::<asset_dynamic_data::Object>(family, object),
        "balance" => deserialize::<balance::Object>(family, object),
        "base" => deserialize::<base::Object>(family, object),
        "blinded_balance" => deserialize::<blinded_balance::Object>(family, object),
        "block_summary" => deserialize::<block_summary::Object>(family, object),
        "budget_record" => deserialize::<budget_record::Object>(family, object),
        "buyback" => deserialize::<buyback::Object>(family, object),
        "call_order" => deserialize::<call_order::Object>(family, object),
        "chain_property" => deserialize::<chain_property::Object>(family, object),
        "collateral_bid" => deserialize::<collateral_bid::Object>(family, object),
        "committee_member" => deserialize::<committee_member::Object>(family, object),
        "credit_deal" => deserialize::<credit_deal::Object>(family, object),
        "credit_deal_summary" => deserialize::<credit_deal_summary::Object>(family, object),
        "credit_offer" => deserialize::<credit_offer::Object>(family, object),
        "custom" => deserialize::<custom::Object>(family, object),
        "custom_authority" => deserialize::<custom_authority::Object>(family, object),
        "dynamic_global_property" => deserialize::<dynamic_global_property::Object>(family, object),
        "fba_accumulator" => deserialize::<fba_accumulator::Object>(family, object),
        "force_settlement" => deserialize::<force_settlement::Object>(family, object),
        "global_property" => deserialize::<global_property::Object>(family, object),
        "htlc" => deserialize::<htlc::Object>(family, object),
        "limit_order" => deserialize::<limit_order::Object>(family, object),
        "liquidity_pool" => deserialize::<liquidity_pool::Object>(family, object),
        "null" => deserialize::<null::Object>(family, object),
        "operation_history" => deserialize::<operation_history::Object>(family, object),
        "proposal" => deserialize::<proposal::Object>(family, object),
        "reserved0" => deserialize::<reserved0::Object>(family, object),
        "samet_fund" => deserialize::<samet_fund::Object>(family, object),
        "special_authority" => deserialize::<special_authority::Object>(family, object),
        "ticket" => deserialize::<ticket::Object>(family, object),
        "transaction_history" => deserialize::<transaction_history::Object>(family, object),
        "vesting_balance" => deserialize::<vesting_balance::Object>(family, object),
        "withdraw_permission" => deserialize::<withdraw_permission::Object>(family, object),
        "witness" => deserialize::<witness::Object>(family, object),
        "witness_schedule" => deserialize::<witness_schedule::Object>(family, object),
        "worker" => deserialize::<worker::Object>(family, object),
        other => panic!("no fixture deserializer is registered for family {other}"),
    }
}

fn deserialize<T>(family: &str, object: Value)
where
    T: DeserializeOwned,
{
    serde_json::from_value::<T>(object)
        .unwrap_or_else(|error| panic!("{family} fixture should deserialize: {error}"));
}
