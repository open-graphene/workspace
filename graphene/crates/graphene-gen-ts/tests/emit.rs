use std::fs;
use std::path::{Path, PathBuf};

use graphene_gen_ts::emit_typescript;
use graphene_spec_gen::lower::lower_document_to_ir;
use graphene_spec_gen::model::OpenGrapheneDocument;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate should live under crates")
        .join("graphene-spec-gen/fixtures/swaplock.opengraphene.json")
}

fn load_swaplock_ir() -> graphene_spec_gen::ir::IrDocument {
    let fixture = fs::read_to_string(fixture_path()).expect("fixture should load from disk");
    let document: OpenGrapheneDocument =
        serde_json::from_str(&fixture).expect("fixture should deserialize");
    lower_document_to_ir(&document).expect("fixture should lower")
}

#[test]
fn emit_typescript_snapshot_from_swaplock_ir() {
    let ir = load_swaplock_ir();
    let files = emit_typescript(&ir).expect("fixture should emit TypeScript");

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path(), Path::new("typescript/index.ts"));
    assert_eq!(
        files[0].contents(),
        include_str!("snapshots/typescript/index.ts")
    );
}

#[test]
fn emit_typescript_errors_include_ir_path_and_type_name() {
    let mut ir = load_swaplock_ir();
    ir.codec_types.get_mut("Asset").unwrap().fields[0].type_ref =
        graphene_spec_gen::ir::IrTypeRef::named("NotAType");

    let error = emit_typescript(&ir).expect_err("unknown type should fail");
    assert_eq!(error.path(), "codecTypes.Asset.fields.amount");
    assert!(error
        .to_string()
        .contains("cannot map IR type 'NotAType' to TypeScript"));
}
