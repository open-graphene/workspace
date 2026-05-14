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

#[test]
fn rejects_invalid_chain_id() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["chain"]["chainId"] = json!("not-a-64-byte-hex-chain-id");

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let errors = validate_document(&document).expect_err("invalid chain id should fail");

    assert!(errors
        .iter()
        .any(|error| { error.path == "chain.chainId" && error.message.contains("32-byte") }));
}

#[test]
fn rejects_duplicate_codec_fields() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["codec"]["types"]["Asset"]["fields"] = json!([
        {
            "name": "amount",
            "type": "int64"
        },
        {
            "name": "amount",
            "type": "object_id"
        }
    ]);

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let errors = validate_document(&document).expect_err("duplicate codec fields should fail");

    assert!(errors.iter().any(|error| {
        error.path == "codec.types.Asset.fields[1].name"
            && error.message.contains("duplicate field name amount")
    }));
}

#[test]
fn rejects_unknown_codec_type_refs() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["codec"]["types"]["Asset"]["fields"][0]["type"] = json!("MissingAmountType");

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let errors = validate_document(&document).expect_err("unknown codec type should fail");

    assert!(errors.iter().any(|error| {
        error.path == "codec.types.Asset.fields[0].type"
            && error.message.contains("MissingAmountType")
    }));
}

#[test]
fn rejects_unknown_callback_api() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["callbacks"]["set_block_applied_callback"]["api"] = json!("missing_callback_api");

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let errors = validate_document(&document).expect_err("unknown callback api should fail");

    assert!(errors.iter().any(|error| {
        error.path == "callbacks.set_block_applied_callback.api"
            && error.message.contains("missing_callback_api")
    }));
}

#[test]
fn rejects_unknown_callback_payload() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["callbacks"]["set_block_applied_callback"]["callbackPayload"] = json!("MissingNotice");

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let errors = validate_document(&document).expect_err("unknown callback payload should fail");

    assert!(errors.iter().any(|error| {
        error.path == "callbacks.set_block_applied_callback.callbackPayload"
            && error.message.contains("MissingNotice")
    }));
}

#[test]
fn rejects_empty_transaction_digest_preimage() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["transaction"]["digest"]["preimage"] = json!([]);

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let errors = validate_document(&document).expect_err("empty digest preimage should fail");

    assert!(errors.iter().any(|error| {
        error.path == "transaction.digest.preimage"
            && error.message.contains("at least one preimage part")
    }));
}

#[test]
fn rejects_duplicate_operation_names() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["operations"]["Operation"] = json!([
        {
            "id": 0,
            "name": "transfer",
            "type": "TransferOperation"
        },
        {
            "id": 1,
            "name": "transfer",
            "type": "TransferOperation"
        }
    ]);

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let errors = validate_document(&document).expect_err("duplicate operation names should fail");

    assert!(errors.iter().any(|error| {
        error.path == "operations.Operation[1].name"
            && error
                .message
                .contains("duplicate operation variant name transfer")
    }));
}
