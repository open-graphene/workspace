use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn fixture_path(name: &str) -> String {
    format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn unique_output_dir(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "open-graphene-gen-cli-{test_name}-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
fn generate_all_writes_expected_files_deterministically() {
    let output_dir = unique_output_dir("generate-all");
    let output_dir_arg = output_dir.to_str().expect("temp path should be UTF-8");

    let run_generate = || {
        Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
            .args([
                "generate",
                &fixture_path("swaplock.opengraphene.json"),
                "--openrpc",
                &fixture_path("swaplock.openrpc.json"),
                "--target",
                "all",
                "--out",
                output_dir_arg,
            ])
            .output()
            .expect("generate should run")
    };

    let first = run_generate();
    assert!(
        first.status.success(),
        "generate failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let typescript_path = output_dir.join("typescript/index.ts");
    let dart_path = output_dir.join("dart/lib/open_graphene.dart");
    assert!(typescript_path.is_file(), "missing TypeScript output");
    assert!(dart_path.is_file(), "missing Dart output");

    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be UTF-8");
    assert!(first_stdout.contains("generated target 'all'"));
    assert!(first_stdout.contains("wrote typescript/index.ts"));
    assert!(first_stdout.contains("wrote dart/lib/open_graphene.dart"));

    let first_typescript = fs::read_to_string(&typescript_path).expect("TypeScript should read");
    let first_dart = fs::read_to_string(&dart_path).expect("Dart should read");

    let second = run_generate();
    assert!(
        second.status.success(),
        "second generate failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    assert_eq!(
        first_stdout,
        String::from_utf8(second.stdout).expect("second stdout should be UTF-8")
    );
    assert_eq!(
        first_typescript,
        fs::read_to_string(&typescript_path).expect("TypeScript should read after rerun")
    );
    assert_eq!(
        first_dart,
        fs::read_to_string(&dart_path).expect("Dart should read after rerun")
    );

    fs::remove_dir_all(output_dir).ok();
}

#[test]
fn conformance_fixtures_writes_transfer_json_deterministically() {
    let output_dir = unique_output_dir("conformance-fixtures");
    let output_dir_arg = output_dir.to_str().expect("temp path should be UTF-8");

    let run_fixtures = || {
        Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
            .args(["conformance-fixtures", "--out", output_dir_arg])
            .output()
            .expect("conformance-fixtures should run")
    };

    let first = run_fixtures();
    assert!(
        first.status.success(),
        "conformance-fixtures failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let fixture_path = output_dir.join("transfer.json");
    assert!(fixture_path.is_file(), "missing transfer fixture");
    let first_json = fs::read_to_string(&fixture_path).expect("fixture should read");
    assert!(!first_json.is_empty(), "fixture should be non-empty");
    let parsed: Value = serde_json::from_str(&first_json).expect("fixture should be JSON");
    assert_eq!(parsed["name"], "swaplock-transfer");
    assert_eq!(
        parsed["expected"]["operationHex"],
        "00400d0300000000000064653930000000000000000000"
    );

    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be UTF-8");
    assert!(first_stdout.contains("generated conformance fixtures into"));
    assert!(first_stdout.contains("wrote transfer.json"));

    let second = run_fixtures();
    assert!(
        second.status.success(),
        "second conformance-fixtures failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        first_stdout,
        String::from_utf8(second.stdout).expect("second stdout should be UTF-8")
    );
    assert_eq!(
        first_json,
        fs::read_to_string(&fixture_path).expect("fixture should read after rerun")
    );

    fs::remove_dir_all(output_dir).ok();
}

#[test]
fn opengraphene_specs_writes_acta_and_swaplock_deterministically() {
    let output_dir = unique_output_dir("opengraphene-specs");
    let output_dir_arg = output_dir.to_str().expect("temp path should be UTF-8");

    let run_specs = || {
        Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
            .args(["opengraphene-specs", "--out", output_dir_arg])
            .output()
            .expect("opengraphene-specs should run")
    };

    let first = run_specs();
    assert!(
        first.status.success(),
        "opengraphene-specs failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let swaplock_path = output_dir.join("swaplock.opengraphene.json");
    let acta_path = output_dir.join("acta.opengraphene.json");
    assert!(
        swaplock_path.is_file(),
        "missing Swaplock OpenGraphene spec"
    );
    assert!(acta_path.is_file(), "missing Acta OpenGraphene spec");

    let first_swaplock = fs::read_to_string(&swaplock_path).expect("Swaplock spec should read");
    let first_acta = fs::read_to_string(&acta_path).expect("Acta spec should read");
    let swaplock: Value =
        serde_json::from_str(&first_swaplock).expect("Swaplock spec should parse");
    let acta: Value = serde_json::from_str(&first_acta).expect("Acta spec should parse");
    assert_eq!(swaplock["chain"]["name"], "swaplock");
    assert_eq!(acta["chain"]["name"], "acta");

    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be UTF-8");
    assert!(first_stdout.contains("generated OpenGraphene specs into"));
    assert!(first_stdout.contains("wrote swaplock.opengraphene.json"));
    assert!(first_stdout.contains("wrote acta.opengraphene.json"));

    let second = run_specs();
    assert!(
        second.status.success(),
        "second opengraphene-specs failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        first_stdout,
        String::from_utf8(second.stdout).expect("second stdout should be UTF-8")
    );
    assert_eq!(
        first_swaplock,
        fs::read_to_string(&swaplock_path).expect("Swaplock spec should read after rerun")
    );
    assert_eq!(
        first_acta,
        fs::read_to_string(&acta_path).expect("Acta spec should read after rerun")
    );

    fs::remove_dir_all(output_dir).ok();
}

#[test]
fn opengraphene_specs_requires_out_directory() {
    let output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args(["opengraphene-specs"])
        .output()
        .expect("opengraphene-specs should run");

    assert!(!output.status.success(), "missing --out should fail");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("opengraphene-specs requires --out <dir>"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn conformance_fixtures_requires_out_directory() {
    let output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args(["conformance-fixtures"])
        .output()
        .expect("conformance-fixtures should run");

    assert!(!output.status.success(), "missing --out should fail");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("conformance-fixtures requires --out <dir>"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generate_rejects_unknown_target() {
    let output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args([
            "generate",
            &fixture_path("swaplock.opengraphene.json"),
            "--target",
            "python",
            "--out",
            unique_output_dir("bad-target")
                .to_str()
                .expect("temp path should be UTF-8"),
        ])
        .output()
        .expect("generate should run");

    assert!(!output.status.success(), "unknown target should fail");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unsupported generate target 'python'"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn inspect_ir_prints_stable_pretty_json_with_openrpc_bindings() {
    let output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args([
            "inspect-ir",
            &fixture_path("swaplock.opengraphene.json"),
            "--openrpc",
            &fixture_path("swaplock.openrpc.json"),
        ])
        .output()
        .expect("inspect-ir should run");

    assert!(
        output.status.success(),
        "inspect-ir failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8 JSON");
    let second_output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args([
            "inspect-ir",
            &fixture_path("swaplock.opengraphene.json"),
            "--openrpc",
            &fixture_path("swaplock.openrpc.json"),
        ])
        .output()
        .expect("second inspect-ir should run");
    assert!(
        second_output.status.success(),
        "second inspect-ir failed: {}",
        String::from_utf8_lossy(&second_output.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8(second_output.stdout).expect("second stdout should be UTF-8 JSON")
    );

    let reparsed: Value = serde_json::from_str(&stdout).expect("inspect-ir should print JSON");

    assert!(reparsed.get("methods").is_some(), "IR missing methods");
    assert!(
        reparsed.get("operations").is_some(),
        "IR missing operations"
    );
    assert!(
        reparsed.get("transaction").is_some(),
        "IR missing transaction"
    );
    assert_eq!(
        reparsed["methods"]["broadcast_transaction"]["params"][0]["name"],
        "SignedTransaction"
    );
    assert_eq!(
        reparsed["methods"]["get_dynamic_global_properties"]["result"]["name"],
        "DynamicGlobalProperties"
    );
}

#[test]
fn inspect_ir_without_openrpc_still_lowers_opengraphene_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args(["inspect-ir", &fixture_path("swaplock.opengraphene.json")])
        .output()
        .expect("inspect-ir should run");

    assert!(
        output.status.success(),
        "inspect-ir failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8 JSON");
    let reparsed: Value = serde_json::from_str(&stdout).expect("inspect-ir should print JSON");

    assert!(reparsed.get("methods").is_some(), "IR missing methods");
    assert!(
        reparsed.get("operations").is_some(),
        "IR missing operations"
    );
    assert!(
        reparsed.get("transaction").is_some(),
        "IR missing transaction"
    );
    assert_eq!(
        reparsed["methods"]["set_block_applied_callback"]["callback"],
        "set_block_applied_callback"
    );
}

#[test]
fn validate_command_still_accepts_existing_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args(["validate", &fixture_path("swaplock.opengraphene.json")])
        .output()
        .expect("validate should run");

    assert!(
        output.status.success(),
        "validate failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("valid OpenGraphene contract"));
}

#[test]
fn validate_command_prints_approved_raw_fallback_warning_without_failing() {
    let contract_path = std::env::temp_dir().join(format!(
        "open-graphene-gen-approved-fallback-{}.json",
        std::process::id()
    ));
    let fixture = fs::read_to_string(fixture_path("swaplock.opengraphene.json"))
        .expect("fixture should read");
    let mut value: Value = serde_json::from_str(&fixture).expect("fixture should parse");
    value["shapeClassifications"] = serde_json::json!([
        {
            "path": "codec.types.TransferOperation.fields[4].type",
            "classification": "approved_raw_fallback",
            "reason": "memo bytes are intentionally retained as raw encrypted payload bytes"
        }
    ]);
    fs::write(
        &contract_path,
        serde_json::to_string_pretty(&value).expect("fixture should serialize"),
    )
    .expect("temp contract should write");

    let output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args([
            "validate",
            contract_path.to_str().expect("temp path should be UTF-8"),
        ])
        .output()
        .expect("validate should run");

    let _ = fs::remove_file(contract_path);

    assert!(
        output.status.success(),
        "approved fallback should not fail: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("valid OpenGraphene contract"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("warning"), "unexpected stderr: {stderr}");
    assert!(
        stderr.contains("approved_raw_fallback"),
        "unexpected stderr: {stderr}"
    );
    assert!(
        stderr.contains("codec.types.TransferOperation.fields[4].type"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn inspect_ir_reports_openrpc_binding_errors() {
    let openrpc_path = std::env::temp_dir().join(format!(
        "open-graphene-gen-openrpc-missing-{}.json",
        std::process::id()
    ));
    fs::write(
        &openrpc_path,
        r##"{
            "openrpc": "1.2.6",
            "methods": [{
                "name": "get_dynamic_global_properties",
                "params": [],
                "result": { "name": "result", "schema": { "$ref": "#/components/schemas/DynamicGlobalProperties" } }
            }],
            "components": { "schemas": { "DynamicGlobalProperties": {}, "void": {} } }
        }"##,
    )
    .expect("temp OpenRPC fixture should be writable");

    let output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args([
            "inspect-ir",
            &fixture_path("swaplock.opengraphene.json"),
            "--openrpc",
            openrpc_path.to_str().expect("temp path should be UTF-8"),
        ])
        .output()
        .expect("inspect-ir should run");

    let _ = fs::remove_file(openrpc_path);

    assert!(
        !output.status.success(),
        "inspect-ir should reject invalid OpenRPC"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("lowering failed with 3 validation errors"),
        "unexpected stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
