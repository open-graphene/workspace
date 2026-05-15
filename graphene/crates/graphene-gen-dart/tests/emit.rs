use std::fs;
use std::path::{Path, PathBuf};

use graphene_gen_dart::emit_dart;
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
fn emit_dart_snapshot_from_swaplock_ir() {
    let ir = load_swaplock_ir();
    let files = emit_dart(&ir).expect("fixture should emit Dart");

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path(), Path::new("dart/lib/open_graphene.dart"));
    assert_eq!(
        files[0].contents(),
        include_str!("snapshots/dart/index.dart")
    );
}

#[test]
fn emit_dart_errors_include_ir_path_and_type_name() {
    let mut ir = load_swaplock_ir();
    ir.codec_types.get_mut("Asset").unwrap().fields[0].type_ref =
        graphene_spec_gen::ir::IrTypeRef::named("NotAType");

    let error = emit_dart(&ir).expect_err("unknown type should fail");
    assert_eq!(error.path(), "codecTypes.Asset.fields.amount");
    assert!(error
        .to_string()
        .contains("cannot map IR type 'NotAType' to Dart"));
}
