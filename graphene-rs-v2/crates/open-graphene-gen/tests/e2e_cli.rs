use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
        "open-graphene-gen-e2e-{test_name}-{}-{nanos}",
        std::process::id()
    ))
}

fn run_cli_stage(stage: &str, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("stage '{stage}' failed to start: {error}"));

    assert!(
        output.status.success(),
        "stage '{stage}' failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    output
}

fn assert_file_contains(stage: &str, path: &Path, expected: &str) {
    let contents = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!(
            "stage '{stage}' expected {} to be readable: {error}",
            path.display()
        )
    });
    assert!(
        contents.contains(expected),
        "stage '{stage}' expected {} to contain {expected:?}\ncontents:\n{contents}",
        path.display()
    );
}

#[test]
fn e2e_cli_workflow_verifies_developer_contract() {
    let opengraphene = fixture_path("swaplock.opengraphene.json");
    let openrpc = fixture_path("swaplock.openrpc.json");
    let output_dir = unique_output_dir("generated");
    let fixture_output_dir = unique_output_dir("conformance");
    let output_dir_arg = output_dir.to_str().expect("temp path should be UTF-8");
    let fixture_output_dir_arg = fixture_output_dir
        .to_str()
        .expect("temp path should be UTF-8");

    let schema = run_cli_stage("schema generation", &["schema"]);
    let schema_stdout =
        String::from_utf8(schema.stdout).expect("schema stdout should be UTF-8 JSON");
    let schema_json: Value = serde_json::from_str(&schema_stdout)
        .expect("stage 'schema generation' should print JSON schema");
    assert_eq!(
        schema_json["title"], "OpenGrapheneDocument",
        "stage 'schema generation' printed an unexpected schema title"
    );
    assert!(
        schema_json["properties"].get("chain").is_some(),
        "stage 'schema generation' schema should include the chain property"
    );

    let validate = run_cli_stage("fixture validation", &["validate", &opengraphene]);
    assert!(
        String::from_utf8_lossy(&validate.stdout).contains("valid OpenGraphene contract"),
        "stage 'fixture validation' did not confirm the fixture was valid"
    );

    let inspect_ir = run_cli_stage(
        "IR inspection",
        &["inspect-ir", &opengraphene, "--openrpc", &openrpc],
    );
    let ir_stdout = String::from_utf8(inspect_ir.stdout).expect("IR stdout should be UTF-8 JSON");
    let ir_json: Value =
        serde_json::from_str(&ir_stdout).expect("stage 'IR inspection' should print JSON IR");
    assert_eq!(
        ir_json["methods"]["broadcast_transaction"]["api"], "network_broadcast",
        "stage 'IR inspection' should include OpenRPC-bound broadcast metadata"
    );
    assert_eq!(
        ir_json["operations"]["Operation"][0]["name"], "transfer",
        "stage 'IR inspection' should include the transfer operation"
    );

    let generate = run_cli_stage(
        "TS/Dart generation",
        &[
            "generate",
            &opengraphene,
            "--openrpc",
            &openrpc,
            "--target",
            "all",
            "--out",
            output_dir_arg,
        ],
    );
    let generate_stdout =
        String::from_utf8(generate.stdout).expect("generate stdout should be UTF-8");
    assert!(
        generate_stdout.contains("generated target 'all'"),
        "stage 'TS/Dart generation' should report the all target"
    );

    let typescript_path = output_dir.join("typescript/index.ts");
    let dart_path = output_dir.join("dart/lib/open_graphene.dart");
    assert!(
        typescript_path.is_file(),
        "stage 'TS/Dart generation' missing {}",
        typescript_path.display()
    );
    assert!(
        dart_path.is_file(),
        "stage 'TS/Dart generation' missing {}",
        dart_path.display()
    );
    assert_file_contains(
        "TS/Dart generation",
        &typescript_path,
        "export const rpcMethods",
    );
    assert_file_contains(
        "TS/Dart generation",
        &dart_path,
        "const Map<String, RpcMethodDescriptor> rpcMethods",
    );

    let conformance = run_cli_stage(
        "conformance fixture emission",
        &["conformance-fixtures", "--out", fixture_output_dir_arg],
    );
    assert!(
        String::from_utf8_lossy(&conformance.stdout)
            .contains("generated conformance fixtures into"),
        "stage 'conformance fixture emission' should report its output directory"
    );

    let transfer_fixture_path = fixture_output_dir.join("transfer.json");
    assert!(
        transfer_fixture_path.is_file(),
        "stage 'conformance fixture emission' missing {}",
        transfer_fixture_path.display()
    );
    let transfer_fixture = fs::read_to_string(&transfer_fixture_path)
        .expect("stage 'conformance fixture emission' transfer fixture should be readable");
    let transfer_fixture_json: Value = serde_json::from_str(&transfer_fixture)
        .expect("stage 'conformance fixture emission' transfer fixture should be JSON");
    assert_eq!(
        transfer_fixture_json["name"], "swaplock-transfer",
        "stage 'conformance fixture emission' wrote an unexpected fixture"
    );
    assert_eq!(
        transfer_fixture_json["expected"]["broadcastPayload"]["method"], "broadcast_transaction",
        "stage 'conformance fixture emission' should include the broadcast payload contract"
    );

    fs::remove_dir_all(output_dir).ok();
    fs::remove_dir_all(fixture_output_dir).ok();
}
