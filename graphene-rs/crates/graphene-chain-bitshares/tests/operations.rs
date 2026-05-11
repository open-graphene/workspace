use graphene_chain_bitshares::operations::{
    BalanceClaimOperation, FillOrderOperation, LimitOrderCreateOperation, Operation,
    TransferOperation,
};
use serde_json::{Value, json};

fn transfer_payload_json() -> Value {
    json!({
        "fee": { "amount": "10", "asset_id": "1.3.0" },
        "from": "1.2.17",
        "to": "1.2.18",
        "amount": { "amount": 100_i64, "asset_id": "1.3.7" },
        "memo": null,
        "extensions": []
    })
}

fn fill_order_payload_json() -> Value {
    json!({
        "fee": { "amount": 0_i64, "asset_id": "1.3.0" },
        "order_id": "1.7.42",
        "account_id": "1.2.17",
        "pays": { "amount": "5000", "asset_id": "1.3.0" },
        "receives": { "amount": 1250_i64, "asset_id": "1.3.7" },
        "fill_price": {
            "base": { "amount": 4_i64, "asset_id": "1.3.0" },
            "quote": { "amount": 1_i64, "asset_id": "1.3.7" }
        },
        "is_maker": true
    })
}

fn balance_claim_payload_json() -> Value {
    json!({
        "balance_owner_key": "BTS8HF8Mtr9TjW1LxQcyP9KWf3BMaj2PCt6HN9YRzysjmrTjbeiE5",
        "balance_to_claim": "1.15.10747",
        "deposit_to_account": "1.2.90744",
        "fee": { "amount": 0_i64, "asset_id": "1.3.0" },
        "total_claimed": { "amount": 81891883_i64, "asset_id": "1.3.0" }
    })
}

#[test]
fn operation_enum_deserializes_typed_static_variant_tags() {
    let transfer: Operation = serde_json::from_value(json!([0_u64, transfer_payload_json()]))
        .expect("tag 0 transfer operation should deserialize through typed enum");
    assert_eq!(transfer.tag(), 0);
    assert!(transfer.is_typed());
    match transfer {
        Operation::Transfer(operation) => {
            assert_eq!(operation.from.to_string(), "1.2.17");
            assert_eq!(operation.amount.asset_id.to_string(), "1.3.7");
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let fill_order: Operation = serde_json::from_value(json!([4_u64, fill_order_payload_json()]))
        .expect("tag 4 fill_order operation should deserialize through typed enum");
    assert_eq!(fill_order.tag(), 4);
    assert!(fill_order.is_typed());
    assert!(matches!(fill_order, Operation::FillOrder(_)));

    let balance_claim: Operation =
        serde_json::from_value(json!([37_u64, balance_claim_payload_json()]))
            .expect("tag 37 balance_claim operation should deserialize through typed enum");
    assert_eq!(balance_claim.tag(), 37);
    assert!(balance_claim.is_typed());
    assert!(matches!(balance_claim, Operation::BalanceClaim(_)));
}

#[test]
fn operation_enum_preserves_skipped_and_unknown_payloads_as_unsupported() {
    let skipped_payload = json!({
        "fee": { "amount": 1_i64, "asset_id": "1.3.0" },
        "registrar": "1.2.17",
        "referrer": "1.2.18",
        "referrer_percent": 0_u64,
        "name": "unsupported-account-create",
        "owner": { "weight_threshold": 1, "account_auths": [], "key_auths": [], "address_auths": [] },
        "active": { "weight_threshold": 1, "account_auths": [], "key_auths": [], "address_auths": [] },
        "options": { "memo_key": "BTS1111111111111111111111111111111114T1Anm", "voting_account": "1.2.5" },
        "extensions": []
    });
    let skipped: Operation = serde_json::from_value(json!([5_u64, skipped_payload.clone()]))
        .expect("unsupported skipped tag should deserialize as explicit fallback");
    assert_eq!(skipped.tag(), 5);
    assert!(!skipped.is_typed());
    assert_eq!(
        skipped,
        Operation::Unsupported {
            tag: 5,
            payload: skipped_payload
        }
    );

    let unknown_payload = json!({ "future": "payload", "nested": [1, 2, 3] });
    let unknown: Operation = serde_json::from_value(json!([999_u64, unknown_payload.clone()]))
        .expect("unknown high tag should deserialize as explicit fallback");
    assert_eq!(unknown.tag(), 999);
    assert!(!unknown.is_typed());
    assert_eq!(
        unknown,
        Operation::Unsupported {
            tag: 999,
            payload: unknown_payload
        }
    );
}

#[test]
fn operation_enum_rejects_malformed_static_variant_shapes() {
    for malformed in [
        json!({ "tag": 0 }),
        json!([]),
        json!([0]),
        json!([0, {}, null]),
    ] {
        let error = serde_json::from_value::<Operation>(malformed)
            .expect_err("operation static_variant must be a two-element array");
        assert!(
            error.to_string().contains("two-element") || error.to_string().contains("array"),
            "unexpected error: {error}"
        );
    }

    let bad_tag = serde_json::from_value::<Operation>(json!(["0", transfer_payload_json()]))
        .expect_err("operation tag must be numeric");
    assert!(
        bad_tag.to_string().contains("tag"),
        "unexpected error: {bad_tag}"
    );

    let bad_payload = serde_json::from_value::<Operation>(json!([0_u64, { "fee": null }]))
        .expect_err("typed operation payload errors should be reported");
    assert!(
        bad_payload.to_string().contains("invalid type") || bad_payload.to_string().contains("fee"),
        "unexpected error: {bad_payload}"
    );
}

#[test]
fn operations_deserialize_transfer_payload_with_protocol_safe_fields() {
    let operation: TransferOperation = serde_json::from_value(transfer_payload_json())
        .expect("transfer operation should deserialize from Graphene payload JSON");

    assert_eq!(operation.fee.amount, 10);
    assert_eq!(operation.fee.asset_id.space(), 1);
    assert_eq!(operation.fee.asset_id.type_id(), 3);
    assert_eq!(operation.fee.asset_id.instance(), 0);
    assert_eq!(operation.from.to_string(), "1.2.17");
    assert_eq!(operation.to.to_string(), "1.2.18");
    assert_eq!(operation.amount.amount, 100);
    assert_eq!(operation.amount.asset_id.to_string(), "1.3.7");
    assert!(operation.memo.is_none());
    assert!(operation.extensions.is_empty());
}

#[test]
fn operations_deserialize_limit_order_create_payload_with_empty_extensions() {
    let operation: LimitOrderCreateOperation = serde_json::from_value(json!({
        "fee": { "amount": 20_i64, "asset_id": "1.3.0" },
        "seller": "1.2.17",
        "amount_to_sell": { "amount": "100000", "asset_id": "1.3.0" },
        "min_to_receive": { "amount": 25000_i64, "asset_id": "1.3.7" },
        "expiration": "2025-01-02T03:04:05",
        "fill_or_kill": false,
        "extensions": []
    }))
    .expect("limit_order_create operation should deserialize from Graphene payload JSON");

    assert_eq!(operation.seller.space(), 1);
    assert_eq!(operation.seller.type_id(), 2);
    assert_eq!(operation.seller.instance(), 17);
    assert_eq!(operation.amount_to_sell.amount, 100000);
    assert_eq!(operation.amount_to_sell.asset_id.to_string(), "1.3.0");
    assert_eq!(operation.min_to_receive.amount, 25000);
    assert_eq!(operation.min_to_receive.asset_id.to_string(), "1.3.7");
    assert_eq!(operation.expiration, "2025-01-02T03:04:05");
    assert!(!operation.fill_or_kill);
    assert!(operation.extensions.is_empty());
}

#[test]
fn operations_deserialize_fill_order_virtual_payload() {
    let operation: FillOrderOperation = serde_json::from_value(fill_order_payload_json())
        .expect("fill_order operation should deserialize from Graphene payload JSON");

    assert_eq!(operation.order_id.to_string(), "1.7.42");
    assert_eq!(operation.account_id.to_string(), "1.2.17");
    assert_eq!(operation.pays.amount, 5000);
    assert_eq!(operation.pays.asset_id.to_string(), "1.3.0");
    assert_eq!(operation.receives.amount, 1250);
    assert_eq!(operation.receives.asset_id.to_string(), "1.3.7");
    assert_eq!(operation.fill_price.base.amount, 4);
    assert_eq!(operation.fill_price.base.asset_id.to_string(), "1.3.0");
    assert_eq!(operation.fill_price.quote.amount, 1);
    assert_eq!(operation.fill_price.quote.asset_id.to_string(), "1.3.7");
    assert!(operation.is_maker);
}

#[test]
fn operations_deserialize_balance_claim_payload_from_tracked_fixture_shape() {
    let operation: BalanceClaimOperation = serde_json::from_value(balance_claim_payload_json())
        .expect("balance_claim operation should deserialize from inline fixture-shaped JSON");

    assert_eq!(operation.fee.amount, 0);
    assert_eq!(operation.fee.asset_id.to_string(), "1.3.0");
    assert_eq!(operation.deposit_to_account.to_string(), "1.2.90744");
    assert_eq!(operation.balance_to_claim.space(), 1);
    assert_eq!(operation.balance_to_claim.type_id(), 15);
    assert_eq!(operation.balance_to_claim.instance(), 10747);
    assert_eq!(
        operation.balance_owner_key,
        "BTS8HF8Mtr9TjW1LxQcyP9KWf3BMaj2PCt6HN9YRzysjmrTjbeiE5"
    );
    assert_eq!(operation.total_claimed.amount, 81891883);
    assert_eq!(operation.total_claimed.asset_id.to_string(), "1.3.0");
}

#[test]
fn operations_deserialize_approved_raw_extension_fallback_shape() {
    let operation: TransferOperation = serde_json::from_value(json!({
        "fee": { "amount": 10_i64, "asset_id": "1.3.0" },
        "from": "1.2.17",
        "to": "1.2.18",
        "amount": { "amount": 100_i64, "asset_id": "1.3.0" },
        "memo": null,
        "extensions": [[0_u64, { "policy": "raw-extension-fixture" }]]
    }))
    .expect("approved raw extension fallback should preserve Graphene static_variant JSON");

    assert_eq!(operation.extensions.len(), 1);
    assert_eq!(
        operation.extensions[0].0,
        json!([0_u64, { "policy": "raw-extension-fixture" }])
    );
}

#[test]
fn operations_reject_malformed_generated_payloads() {
    let bad_object_id = serde_json::from_value::<TransferOperation>(json!({
        "fee": { "amount": 10_i64, "asset_id": "1.3.0" },
        "from": "1.2",
        "to": "1.2.18",
        "amount": { "amount": 100_i64, "asset_id": "1.3.0" },
        "memo": null,
        "extensions": []
    }))
    .expect_err("malformed object id should be rejected");
    assert!(
        bad_object_id
            .to_string()
            .contains("object id must have space.type.instance parts"),
        "unexpected error: {bad_object_id}"
    );

    let missing_required_field = serde_json::from_value::<LimitOrderCreateOperation>(json!({
        "fee": { "amount": 20_i64, "asset_id": "1.3.0" },
        "seller": "1.2.17",
        "amount_to_sell": { "amount": 100000_i64, "asset_id": "1.3.0" },
        "expiration": "2025-01-02T03:04:05",
        "fill_or_kill": false,
        "extensions": []
    }))
    .expect_err("missing min_to_receive should be rejected");
    assert!(
        missing_required_field
            .to_string()
            .contains("min_to_receive"),
        "unexpected error: {missing_required_field}"
    );
}

#[test]
fn operations_report_names_raw_fallback_and_unsupported_metadata() {
    let report = include_str!("../src/operation_model_skips.md");

    assert!(report.contains("| transfer_operation | 0 | extensions | `extensions_type` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/transfer.hpp:62 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |"));
    assert!(report.contains("| account_create_operation | 5 | options | `account_options` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/account.hpp:111 | unsupported | no protocol-safe operation mapping for C++ type account_options |"));
    assert!(report.contains("| limit_order_update_operation | 77 | on_fill | `optional<vector<limit_order_auto_action>>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/market.hpp:129 | unsupported | no protocol-safe operation mapping for C++ type optional<vector<limit_order_auto_action>> |"));
}
