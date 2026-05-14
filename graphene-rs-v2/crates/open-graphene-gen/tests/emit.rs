use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use open_graphene_gen::emit::typescript::emit_typescript;
use open_graphene_gen::emit::{write_generated_files, GeneratedFile};
use open_graphene_gen::lower::lower_document_to_ir;
use open_graphene_gen::model::OpenGrapheneDocument;

fn unique_output_dir(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "open-graphene-gen-{test_name}-{}-{nanos}",
        std::process::id()
    ))
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/swaplock.opengraphene.json")
}

fn load_swaplock_ir() -> open_graphene_gen::ir::IrDocument {
    let fixture = fs::read_to_string(fixture_path()).expect("fixture should load from disk");
    let document: OpenGrapheneDocument =
        serde_json::from_str(&fixture).expect("fixture should deserialize");
    lower_document_to_ir(&document).expect("fixture should lower")
}

#[test]
fn emit_writes_generated_files_in_deterministic_order_and_overwrites() {
    let output_dir = unique_output_dir("deterministic-output");
    let files = vec![
        GeneratedFile::new("zeta.txt", "old zeta").unwrap(),
        GeneratedFile::new("nested/alpha.txt", "alpha").unwrap(),
        GeneratedFile::new("zeta.txt", "new zeta").unwrap(),
    ];

    let written = write_generated_files(&output_dir, &files).expect("files should write");
    let paths = written
        .iter()
        .map(|file| file.path.as_path())
        .collect::<Vec<_>>();

    assert_eq!(
        paths,
        vec![
            Path::new("nested/alpha.txt"),
            Path::new("zeta.txt"),
            Path::new("zeta.txt")
        ]
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("nested/alpha.txt")).unwrap(),
        "alpha"
    );
    assert_eq!(
        fs::read_to_string(output_dir.join("zeta.txt")).unwrap(),
        "new zeta"
    );

    fs::remove_dir_all(output_dir).ok();
}

#[test]
fn emit_rejects_paths_outside_output_directory() {
    let error = GeneratedFile::new("../escape.txt", "bad").expect_err("parent paths fail");
    assert_eq!(error.path(), "../escape.txt");
    assert!(error
        .to_string()
        .contains("stay inside the output directory"));
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
        open_graphene_gen::ir::IrTypeRef::named("NotAType");

    let error = emit_typescript(&ir).expect_err("unknown type should fail");
    assert_eq!(error.path(), "codecTypes.Asset.fields.amount");
    assert!(error
        .to_string()
        .contains("cannot map IR type 'NotAType' to TypeScript"));
}
