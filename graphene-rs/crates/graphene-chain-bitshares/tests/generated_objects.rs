use graphene_chain_bitshares::types::{
    account_balance, account_history, asset_dynamic_data, block_summary, buyback, committee_member,
    dynamic_global_property, fba_accumulator,
};
use serde_json::json;

#[test]
fn deserializes_account_balance_object() {
    let object: account_balance::Object = serde_json::from_value(json!({
        "id": "2.5.42",
        "owner": "1.2.7",
        "asset_type": "1.3.0",
        "balance": 123456789_i64,
        "maintenance_flag": false
    }))
    .expect("account_balance object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.5.42");
    assert_eq!(object.id.instance(), 42);
    assert_eq!(object.owner.to_string(), "1.2.7");
    assert_eq!(object.asset_type.to_string(), "1.3.0");
    assert_eq!(object.balance, 123456789);
    assert!(!object.maintenance_flag);
}

#[test]
fn deserializes_account_history_object() {
    let object: account_history::Object = serde_json::from_value(json!({
        "id": "2.9.100",
        "account": "1.2.17",
        "operation_id": "1.11.55",
        "sequence": 7_u64,
        "next": "2.9.99"
    }))
    .expect("account_history object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.9.100");
    assert_eq!(object.account.to_string(), "1.2.17");
    assert_eq!(object.operation_id.to_string(), "1.11.55");
    assert_eq!(object.sequence, 7);
    assert_eq!(object.next.to_string(), "2.9.99");
}

#[test]
fn deserializes_asset_dynamic_data_object() {
    let object: asset_dynamic_data::Object = serde_json::from_value(json!({
        "id": "2.3.0",
        "current_supply": 1_000_000_i64,
        "confidential_supply": 0_i64,
        "accumulated_fees": 10_i64,
        "accumulated_collateral_fees": 20_i64,
        "fee_pool": 30_i64
    }))
    .expect("asset_dynamic_data object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.3.0");
    assert_eq!(object.current_supply, 1_000_000);
    assert_eq!(object.confidential_supply, 0);
    assert_eq!(object.accumulated_fees, 10);
    assert_eq!(object.accumulated_collateral_fees, 20);
    assert_eq!(object.fee_pool, 30);
}

#[test]
fn deserializes_block_summary_object() {
    let block_id = "0000000a4f3d2c1b000000000000000000000000";
    let object: block_summary::Object = serde_json::from_value(json!({
        "id": "2.8.10",
        "block_id": block_id
    }))
    .expect("block_summary object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.8.10");
    assert_eq!(object.block_id, block_id);
}

#[test]
fn deserializes_buyback_object() {
    let object: buyback::Object = serde_json::from_value(json!({
        "id": "2.15.4",
        "asset_to_buy": "1.3.7"
    }))
    .expect("buyback object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.15.4");
    assert_eq!(object.asset_to_buy.to_string(), "1.3.7");
}

#[test]
fn deserializes_committee_member_object() {
    let object: committee_member::Object = serde_json::from_value(json!({
        "id": "1.5.2",
        "committee_member_account": "1.2.17",
        "vote_id": "0:12",
        "total_votes": 88_u64,
        "url": "https://committee.example"
    }))
    .expect("committee_member object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.5.2");
    assert_eq!(object.committee_member_account.to_string(), "1.2.17");
    assert_eq!(object.vote_id, "0:12");
    assert_eq!(object.total_votes, 88);
    assert_eq!(object.url, "https://committee.example");
}

#[test]
fn deserializes_dynamic_global_property_object() {
    let recent_slots_filled = 123456789_u128;
    let object: dynamic_global_property::Object = serde_json::from_value(json!({
        "id": "2.1.0",
        "head_block_number": 123_u32,
        "head_block_id": "0000007b4f3d2c1b000000000000000000000000",
        "time": "2024-01-02T03:04:05",
        "current_witness": "1.6.5",
        "next_maintenance_time": "2024-01-02T04:00:00",
        "last_vote_tally_time": "2024-01-01T00:00:00",
        "last_budget_time": "2024-01-02T02:00:00",
        "witness_budget": -10_i64,
        "total_pob": 20_i64,
        "total_inactive": 30_i64,
        "accounts_registered_this_interval": 4_u32,
        "recently_missed_count": 5_u32,
        "current_aslot": 6_u64,
        "recent_slots_filled": recent_slots_filled,
        "dynamic_flags": 1_u32,
        "last_irreversible_block_num": 122_u32
    }))
    .expect("dynamic_global_property object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.1.0");
    assert_eq!(object.head_block_number, 123);
    assert_eq!(
        object.head_block_id,
        "0000007b4f3d2c1b000000000000000000000000"
    );
    assert_eq!(object.time, "2024-01-02T03:04:05");
    assert_eq!(object.current_witness.to_string(), "1.6.5");
    assert_eq!(object.next_maintenance_time, "2024-01-02T04:00:00");
    assert_eq!(object.last_vote_tally_time, "2024-01-01T00:00:00");
    assert_eq!(object.last_budget_time, "2024-01-02T02:00:00");
    assert_eq!(object.witness_budget, -10);
    assert_eq!(object.total_pob, 20);
    assert_eq!(object.total_inactive, 30);
    assert_eq!(object.accounts_registered_this_interval, 4);
    assert_eq!(object.recently_missed_count, 5);
    assert_eq!(object.current_aslot, 6);
    assert_eq!(object.recent_slots_filled, recent_slots_filled);
    assert_eq!(object.dynamic_flags, 1);
    assert_eq!(object.last_irreversible_block_num, 122);
}

#[test]
fn deserializes_fba_accumulator_object_with_designated_asset() {
    let object: fba_accumulator::Object = serde_json::from_value(json!({
        "id": "2.16.3",
        "accumulated_fba_fees": 999_i64,
        "designated_asset": "1.3.7"
    }))
    .expect("fba_accumulator object should deserialize with designated asset");

    assert_eq!(object.id.to_string(), "2.16.3");
    assert_eq!(object.accumulated_fba_fees, 999);
    assert_eq!(
        object
            .designated_asset
            .expect("asset id should be present")
            .to_string(),
        "1.3.7"
    );
}

#[test]
fn deserializes_fba_accumulator_object_without_designated_asset() {
    let object: fba_accumulator::Object = serde_json::from_value(json!({
        "id": "2.16.4",
        "accumulated_fba_fees": -5_i64,
        "designated_asset": null
    }))
    .expect("fba_accumulator object should deserialize without designated asset");

    assert_eq!(object.id.to_string(), "2.16.4");
    assert_eq!(object.accumulated_fba_fees, -5);
    assert!(object.designated_asset.is_none());
}

#[test]
fn rejects_wrong_object_family_id() {
    let error = serde_json::from_value::<account_balance::Object>(json!({
        "id": "2.3.42",
        "owner": "1.2.7",
        "asset_type": "1.3.0",
        "balance": 123_i64,
        "maintenance_flag": false
    }))
    .expect_err("account_balance id should reject asset_dynamic_data object family");

    assert!(
        error.to_string().contains("not a account balance id"),
        "unexpected error: {error}"
    );
}
