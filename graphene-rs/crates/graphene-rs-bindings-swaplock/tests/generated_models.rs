use graphene_rs_bindings_swaplock::{Operation, TransferOperation, OPERATIONS};
use serde_json::json;

fn transfer_payload() -> serde_json::Value {
    json!({
        "fee": { "amount": 0, "asset_id": "1.3.0" },
        "from": "1.2.100",
        "to": "1.2.101",
        "amount": { "amount": 42, "asset_id": "1.3.0" },
        "memo": null,
        "extensions": []
    })
}

#[test]
fn operation_metadata_contains_transfer_mapping() {
    let transfer = OPERATIONS
        .iter()
        .find(|operation| operation.name == "transfer")
        .expect("transfer operation metadata");

    assert_eq!(transfer.id, 0);
    assert_eq!(transfer.cpp_type, "transfer_operation");
    assert_eq!(transfer.rust_type, "TransferOperation");
    assert_eq!(transfer.schema, "#/components/schemas/transfer_operation");
}

#[test]
fn transfer_operation_payload_round_trips_through_json() {
    let payload = transfer_payload();
    let transfer: TransferOperation = serde_json::from_value(payload.clone()).unwrap();
    let encoded = serde_json::to_value(&transfer).unwrap();

    assert_eq!(encoded, payload);
}

#[test]
fn operation_serializes_as_static_variant_index_payload_tuple() {
    let transfer: TransferOperation = serde_json::from_value(transfer_payload()).unwrap();
    let operation = Operation::Transfer(transfer);

    let encoded = serde_json::to_value(&operation).unwrap();

    assert_eq!(encoded[0], json!(0));
    assert_eq!(encoded[1]["from"], json!("1.2.100"));
    assert_eq!(encoded[1]["to"], json!("1.2.101"));
    assert_eq!(encoded[1]["amount"]["amount"], json!(42));
}

#[test]
fn operation_deserializes_from_static_variant_index_payload_tuple() {
    let encoded = json!([0, transfer_payload()]);
    let operation: Operation = serde_json::from_value(encoded).unwrap();

    match operation {
        Operation::Transfer(transfer) => {
            let payload = serde_json::to_value(transfer).unwrap();
            assert_eq!(payload["from"], json!("1.2.100"));
            assert_eq!(payload["amount"]["asset_id"], json!("1.3.0"));
        }
        _ => panic!("expected transfer operation"),
    }
}
