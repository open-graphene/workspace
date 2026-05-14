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

fn assert_cli_fixture_contract(
    chain_name: &str,
    opengraphene: &str,
    openrpc: &str,
    expected_method: &str,
    expected_result: Option<&str>,
    expected_callback: &str,
) {
    let validate = run_cli_stage(
        &format!("{chain_name} fixture validation"),
        &["validate", opengraphene],
    );
    let validate_stdout = String::from_utf8_lossy(&validate.stdout);
    assert!(
        validate_stdout.contains("valid OpenGraphene contract"),
        "stage '{chain_name} fixture validation' did not confirm the fixture was valid"
    );

    let inspect_ir = run_cli_stage(
        &format!("{chain_name} IR inspection"),
        &["inspect-ir", opengraphene, "--openrpc", openrpc],
    );
    let ir_stdout = String::from_utf8(inspect_ir.stdout)
        .unwrap_or_else(|error| panic!("{chain_name} IR stdout should be UTF-8 JSON: {error}"));
    let ir_json: Value = serde_json::from_str(&ir_stdout).unwrap_or_else(|error| {
        panic!("stage '{chain_name} IR inspection' should print JSON IR: {error}")
    });

    assert_eq!(
        ir_json["chain"]["name"], chain_name,
        "stage '{chain_name} IR inspection' should preserve chain metadata"
    );
    assert_eq!(
        ir_json["methods"][expected_method]["api"], "network_broadcast",
        "stage '{chain_name} IR inspection' should include OpenRPC-bound broadcast metadata"
    );
    if let Some(result) = expected_result {
        assert_eq!(
            ir_json["methods"][expected_method]["result"]["name"], result,
            "stage '{chain_name} IR inspection' should include expected method result metadata"
        );
    }
    assert_eq!(
        ir_json["operations"]["Operation"][0]["name"], "transfer",
        "stage '{chain_name} IR inspection' should include the transfer operation"
    );
    assert!(
        ir_json["callbacks"].get(expected_callback).is_some(),
        "stage '{chain_name} IR inspection' should include expected callback metadata"
    );
}

#[test]
fn e2e_cli_workflow_verifies_developer_contract() {
    let swaplock_opengraphene = fixture_path("swaplock.opengraphene.json");
    let swaplock_openrpc = fixture_path("swaplock.openrpc.json");
    let acta_opengraphene = fixture_path("acta.opengraphene.json");
    let acta_openrpc = fixture_path("acta.openrpc.json");
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

    assert_cli_fixture_contract(
        "swaplock",
        &swaplock_opengraphene,
        &swaplock_openrpc,
        "broadcast_transaction",
        Some("void"),
        "set_block_applied_callback",
    );
    assert_cli_fixture_contract(
        "acta",
        &acta_opengraphene,
        &acta_openrpc,
        "broadcast_transaction_synchronous",
        Some("SynchronousBroadcastResult"),
        "set_subscribe_callback",
    );

    let generate = run_cli_stage(
        "prototype TS/Dart metadata generation",
        &[
            "generate",
            &swaplock_opengraphene,
            "--openrpc",
            &swaplock_openrpc,
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
        "stage 'prototype TS/Dart metadata generation' should report the all target"
    );

    let typescript_path = output_dir.join("typescript/index.ts");
    let dart_path = output_dir.join("dart/lib/open_graphene.dart");
    assert!(
        typescript_path.is_file(),
        "stage 'prototype TS/Dart metadata generation' missing {}",
        typescript_path.display()
    );
    assert!(
        dart_path.is_file(),
        "stage 'prototype TS/Dart metadata generation' missing {}",
        dart_path.display()
    );
    assert_file_contains(
        "prototype TS/Dart metadata generation",
        &typescript_path,
        "export const rpcMethods",
    );
    assert_file_contains(
        "prototype TS/Dart metadata generation",
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
