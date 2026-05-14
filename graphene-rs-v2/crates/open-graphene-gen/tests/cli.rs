use std::fs;
use std::process::Command;

use serde_json::Value;

fn fixture_path(name: &str) -> String {
    format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
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
