use std::fs;
use std::path::PathBuf;

use open_graphene_gen::ir::{IrCallbackLifetime, IrTypeRef};
use open_graphene_gen::lower::lower_document_to_ir;
use open_graphene_gen::model::OpenGrapheneDocument;
use serde_json::json;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/swaplock.opengraphene.json")
}

fn load_swaplock_fixture() -> OpenGrapheneDocument {
    let fixture = fs::read_to_string(fixture_path()).expect("fixture should load from disk");
    serde_json::from_str(&fixture).expect("fixture should deserialize")
}

#[test]
fn lower_document_to_ir_resolves_swaplock_fixture() {
    let document = load_swaplock_fixture();
    let ir = lower_document_to_ir(&document).expect("fixture should lower");

    assert_eq!(ir.contract_version, "0.1");
    assert_eq!(
        ir.openrpc_source.as_deref(),
        Some("./swaplock.openrpc.json")
    );
    assert_eq!(ir.chain.name, "swaplock");
    assert_eq!(
        ir.apis.keys().cloned().collect::<Vec<_>>(),
        vec!["database", "network_broadcast"]
    );
    assert_eq!(
        ir.methods.keys().cloned().collect::<Vec<_>>(),
        vec![
            "broadcast_transaction",
            "broadcast_transaction_with_callback",
            "get_dynamic_global_properties",
            "set_block_applied_callback"
        ]
    );
    assert_eq!(
        ir.methods["broadcast_transaction_with_callback"].params,
        vec![IrTypeRef::named("SignedTransaction")]
    );
    assert_eq!(
        ir.methods["broadcast_transaction_with_callback"].result,
        Some(IrTypeRef::named("void"))
    );
    assert_eq!(
        ir.codec_types["TransferOperation"].fields[4].type_ref,
        IrTypeRef::optional(IrTypeRef::named("bytes"))
    );
    assert_eq!(ir.operation_variants["Operation"][0].name, "transfer");
    assert_eq!(
        ir.transaction.as_ref().unwrap().operation_variant,
        "Operation"
    );
    assert_eq!(
        ir.callbacks["set_block_applied_callback"].callback_lifetime,
        IrCallbackLifetime::Persistent
    );
    assert!(ir.diagnostics.is_empty());
}

#[test]
fn lower_document_to_ir_returns_path_specific_validation_errors() {
    let mut value = serde_json::to_value(load_swaplock_fixture()).unwrap();
    value["callbacks"]["set_block_applied_callback"]["callbackPayload"] = json!("MissingNotice");
    value["codec"]["types"]["Asset"]["fields"][0]["type"] = json!("MissingAmountType");

    let document: OpenGrapheneDocument = serde_json::from_value(value).unwrap();
    let error = lower_document_to_ir(&document).expect_err("invalid contract should not lower");
    let paths = error
        .errors()
        .iter()
        .map(|error| error.path.as_str())
        .collect::<Vec<_>>();

    assert!(paths.contains(&"callbacks.set_block_applied_callback.callbackPayload"));
    assert!(paths.contains(&"codec.types.Asset.fields[0].type"));
    assert!(error
        .to_string()
        .contains("lowering failed with 2 validation errors"));
}
