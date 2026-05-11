use graphene_chain_bitshares::types::{account_balance, asset_dynamic_data, fba_accumulator};
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
