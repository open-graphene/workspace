use graphene_chain_bitshares::types::{
    account_balance, account_history, asset_dynamic_data, blinded_balance, block_summary, buyback,
    committee_member, dynamic_global_property, fba_accumulator, withdraw_permission, witness,
    witness_schedule,
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
fn deserializes_blinded_balance_object() {
    let commitment = "028f7d2c1b00000000000000000000000000000000000000000000000000000000";
    let key = "BTS1111111111111111111111111111111114T1Anm";
    let address = "BTSFN9r6VYzBK8EKtMewfNbfiGCr56pHDBFi";
    let object: blinded_balance::Object = serde_json::from_value(json!({
        "id": "2.10.4",
        "commitment": commitment,
        "asset_id": "1.3.0",
        "owner": {
            "weight_threshold": 2_u32,
            "account_auths": [["1.2.17", 1_u16]],
            "key_auths": [[key, 1_u16]],
            "address_auths": [[address, 1_u16]]
        }
    }))
    .expect("blinded_balance object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.10.4");
    assert_eq!(object.commitment, commitment);
    assert_eq!(object.asset_id.to_string(), "1.3.0");
    assert_eq!(object.owner.weight_threshold, 2);
    assert_eq!(object.owner.account_auths[0].0.to_string(), "1.2.17");
    assert_eq!(object.owner.account_auths[0].1, 1);
    assert_eq!(object.owner.key_auths, vec![(key.to_owned(), 1)]);
    assert_eq!(object.owner.address_auths, vec![(address.to_owned(), 1)]);
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
fn deserializes_witness_object() {
    let signing_key = "BTS1111111111111111111111111111111114T1Anm";
    let object: witness::Object = serde_json::from_value(json!({
        "id": "1.6.5",
        "witness_account": "1.2.17",
        "last_aslot": 123_u64,
        "signing_key": signing_key,
        "pay_vb": "1.13.8",
        "vote_id": "1:5",
        "total_votes": 456_u64,
        "url": "https://witness.example",
        "total_missed": -1_i64,
        "last_confirmed_block_num": 789_u32
    }))
    .expect("witness object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.6.5");
    assert_eq!(object.witness_account.to_string(), "1.2.17");
    assert_eq!(object.last_aslot, 123);
    assert_eq!(object.signing_key, signing_key);
    assert_eq!(
        object
            .pay_vb
            .expect("pay vesting balance should be present")
            .to_string(),
        "1.13.8"
    );
    assert_eq!(object.vote_id, "1:5");
    assert_eq!(object.total_votes, 456);
    assert_eq!(object.url, "https://witness.example");
    assert_eq!(object.total_missed, -1);
    assert_eq!(object.last_confirmed_block_num, 789);
}

#[test]
fn deserializes_witness_schedule_object() {
    let object: witness_schedule::Object = serde_json::from_value(json!({
        "id": "2.12.0",
        "current_shuffled_witnesses": ["1.6.5", "1.6.6"]
    }))
    .expect("witness_schedule object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.12.0");
    let witness_ids = object
        .current_shuffled_witnesses
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(witness_ids, vec!["1.6.5", "1.6.6"]);
}

#[test]
fn deserializes_withdraw_permission_object() {
    let object: withdraw_permission::Object = serde_json::from_value(json!({
        "id": "1.12.9",
        "withdraw_from_account": "1.2.17",
        "authorized_account": "1.2.18",
        "withdrawal_limit": {
            "amount": 5000_i64,
            "asset_id": "1.3.0"
        },
        "withdrawal_period_sec": 3600_u32,
        "period_start_time": "2024-01-02T03:04:05",
        "expiration": "2024-02-02T03:04:05",
        "claimed_this_period": 250_i64
    }))
    .expect("withdraw_permission object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.12.9");
    assert_eq!(object.withdraw_from_account.to_string(), "1.2.17");
    assert_eq!(object.authorized_account.to_string(), "1.2.18");
    assert_eq!(object.withdrawal_limit.amount, 5000);
    assert_eq!(object.withdrawal_limit.asset_id.to_string(), "1.3.0");
    assert_eq!(object.withdrawal_period_sec, 3600);
    assert_eq!(object.period_start_time, "2024-01-02T03:04:05");
    assert_eq!(object.expiration, "2024-02-02T03:04:05");
    assert_eq!(object.claimed_this_period, 250);
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
