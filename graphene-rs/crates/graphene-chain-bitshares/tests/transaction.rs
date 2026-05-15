use graphene_chain_bitshares::operations::Operation;
use graphene_chain_bitshares::transaction::{OperationResult, SignedTransaction, Transaction};
use serde_json::{Value, json};

fn transfer_operation_json() -> Value {
    json!([
        0_u64,
        {
            "fee": { "amount": "10", "asset_id": "1.3.0" },
            "from": "1.2.17",
            "to": "1.2.18",
            "amount": { "amount": 100_i64, "asset_id": "1.3.7" },
            "memo": null,
            "extensions": []
        }
    ])
}

fn transaction_json() -> Value {
    json!({
        "ref_block_num": 12345_u64,
        "ref_block_prefix": 987654321_u64,
        "expiration": "2025-01-02T03:04:05",
        "operations": [transfer_operation_json()],
        "extensions": []
    })
}

#[test]
fn transaction_deserializes_inline_operations_through_bitshares_operation_model() {
    let transaction: Transaction = serde_json::from_value(transaction_json()).expect(
        "transaction should deserialize inline operations through typed BitShares Operation",
    );

    assert_eq!(transaction.ref_block_num, 12345);
    assert_eq!(transaction.ref_block_prefix, 987654321);
    assert_eq!(transaction.expiration, "2025-01-02T03:04:05");
    assert_eq!(transaction.extensions.len(), 0);
    assert_eq!(transaction.operations.len(), 1);
    assert!(matches!(transaction.operations[0], Operation::Transfer(_)));
}

#[test]
fn signed_transaction_deserializes_signatures_and_nested_operations() {
    let signed: SignedTransaction = serde_json::from_value(json!({
        "ref_block_num": 12345_u64,
        "ref_block_prefix": 987654321_u64,
        "expiration": "2025-01-02T03:04:05",
        "operations": [transfer_operation_json()],
        "extensions": [[0_u64, { "raw_extension": true }]],
        "signatures": ["1f5c7a"]
    }))
    .expect("signed transaction should deserialize transaction fields plus string signatures");

    assert_eq!(signed.transaction.operations.len(), 1);
    assert_eq!(
        signed.transaction.extensions[0].0,
        json!([0_u64, { "raw_extension": true }])
    );
    assert_eq!(signed.signatures, vec!["1f5c7a".to_string()]);
}

#[test]
fn transaction_operation_result_deserializes_void_and_preserves_unsupported_payloads() {
    let void: OperationResult = serde_json::from_value(json!([0_u64, {}]))
        .expect("void operation result should deserialize from [0, {}]");
    assert_eq!(void.tag(), 0);
    assert!(void.is_typed());
    assert!(matches!(void, OperationResult::Void));

    let payload = json!({ "future_result": "payload", "nested": [1, 2, 3] });
    let unsupported: OperationResult = serde_json::from_value(json!([999_u64, payload.clone()]))
        .expect("unsupported operation result tag should preserve payload");
    assert_eq!(unsupported.tag(), 999);
    assert!(!unsupported.is_typed());
    assert_eq!(
        unsupported,
        OperationResult::Unsupported { tag: 999, payload }
    );
}

#[test]
fn transaction_rejects_malformed_nested_operations_and_missing_fields() {
    let missing_operations = serde_json::from_value::<Transaction>(json!({
        "ref_block_num": 12345_u64,
        "ref_block_prefix": 987654321_u64,
        "expiration": "2025-01-02T03:04:05",
        "extensions": []
    }))
    .expect_err("missing operations should be rejected");
    assert!(
        missing_operations.to_string().contains("operations"),
        "unexpected error: {missing_operations}"
    );

    let bad_nested_operation = serde_json::from_value::<Transaction>(json!({
        "ref_block_num": 12345_u64,
        "ref_block_prefix": 987654321_u64,
        "expiration": "2025-01-02T03:04:05",
        "operations": [[0_u64, { "fee": null }]],
        "extensions": []
    }))
    .expect_err("malformed nested operation payload should bubble serde error");
    assert!(
        bad_nested_operation.to_string().contains("invalid type")
            || bad_nested_operation.to_string().contains("fee"),
        "unexpected error: {bad_nested_operation}"
    );
}

#[test]
fn transaction_rejects_wrong_static_variant_shapes_and_bad_signature_types() {
    for malformed_result in [
        json!({ "tag": 0 }),
        json!([]),
        json!([0]),
        json!([0, {}, null]),
    ] {
        let error = serde_json::from_value::<OperationResult>(malformed_result)
            .expect_err("operation result static_variant must be a two-element array");
        assert!(
            error.to_string().contains("two-element") || error.to_string().contains("array"),
            "unexpected error: {error}"
        );
    }

    let bad_result_tag = serde_json::from_value::<OperationResult>(json!(["0", {}]))
        .expect_err("operation result tag must be numeric");
    assert!(
        bad_result_tag.to_string().contains("tag"),
        "unexpected error: {bad_result_tag}"
    );

    let bad_signature_type = serde_json::from_value::<SignedTransaction>(json!({
        "ref_block_num": 12345_u64,
        "ref_block_prefix": 987654321_u64,
        "expiration": "2025-01-02T03:04:05",
        "operations": [transfer_operation_json()],
        "extensions": [],
        "signatures": [123]
    }))
    .expect_err("signatures must be strings");
    assert!(
        bad_signature_type.to_string().contains("string"),
        "unexpected error: {bad_signature_type}"
    );
}
