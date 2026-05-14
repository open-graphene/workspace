use std::fs;
use std::path::PathBuf;

use open_graphene_gen::model::OpenGrapheneDocument;
use open_graphene_gen::validation::validate_document;
use serde_json::json;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/swaplock.opengraphene.json")
}

fn load_swaplock_fixture() -> OpenGrapheneDocument {
    let fixture = fs::read_to_string(fixture_path()).expect("fixture should load from disk");
    serde_json::from_str(&fixture).expect("fixture should deserialize")
}

#[test]
fn accepts_minimal_valid_contract() {
    let document = load_swaplock_fixture();
    validate_document(&document).expect("valid contract should pass");
}

#[test]
fn rejects_unknown_references() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["methodBindings"]["get_dynamic_global_properties"]["api"] = json!("missing");
    value["transaction"]["operationVariant"] = json!("MissingOperation");

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let errors = validate_document(&document).expect_err("invalid contract should fail");

    assert!(errors
        .iter()
        .any(|error| error.path == "methodBindings.get_dynamic_global_properties.api"));
    assert!(errors
        .iter()
        .any(|error| error.path == "transaction.operationVariant"));
}

#[test]
fn rejects_duplicate_operation_ids() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["operations"]["Operation"] = json!([
        {
            "id": 0,
            "name": "transfer",
            "type": "TransferOperation"
        },
        {
            "id": 0,
            "name": "transfer_to_blind",
            "type": "TransferOperation"
        }
    ]);

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let errors = validate_document(&document).expect_err("duplicate variant should fail");

    assert!(errors
        .iter()
        .any(|error| error.path == "operations.Operation[1].id"));
}
