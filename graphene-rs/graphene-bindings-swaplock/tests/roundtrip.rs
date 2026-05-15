use graphene_bindings_swaplock::{Operation, TransferOperation};
use serde_json::json;

#[test]
fn transfer_operation_round_trips_through_operation_envelope() {
    let payload_json = json!({
        "fee":  {"amount": 100, "asset_id": "1.3.0"},
        "from": "1.2.42",
        "to":   "1.2.99",
        "amount": {"amount": 5000, "asset_id": "1.3.0"},
        "memo": null,
        "extensions": [],
    });
    let payload: TransferOperation = serde_json::from_value(payload_json.clone()).unwrap();
    let operation = Operation::Transfer(payload);

    let encoded = serde_json::to_value(&operation).unwrap();
    assert_eq!(encoded[0], json!(0), "transfer's wire index is 0");
    assert_eq!(encoded[1], payload_json);

    let decoded: Operation = serde_json::from_value(encoded).unwrap();
    assert!(matches!(decoded, Operation::Transfer(_)));
}

#[test]
fn unknown_operation_index_is_a_clean_error() {
    let raw = json!([999, {}]);
    let error = serde_json::from_value::<Operation>(raw).unwrap_err();
    assert!(error
        .to_string()
        .contains("unknown Operation variant index"));
}
