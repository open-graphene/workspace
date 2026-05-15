use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should live under crates/graphene-gen-ts")
        .to_path_buf()
}

fn unique_output_dir(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "graphene-gen-ts-{test_name}-{}-{nanos}",
        std::process::id()
    ))
}

fn run_generate(output_dir: &Path) -> std::process::Output {
    let workspace = workspace_root();
    Command::new(env!("CARGO_BIN_EXE_graphene-gen-ts"))
        .args([
            "generate",
            workspace
                .join("specs/swaplock.opengraphene.json")
                .to_str()
                .expect("path should be UTF-8"),
            "--openrpc",
            workspace
                .join("crates/graphene-spec-gen/fixtures/swaplock.openrpc.json")
                .to_str()
                .expect("path should be UTF-8"),
            "--out",
            output_dir.to_str().expect("path should be UTF-8"),
        ])
        .output()
        .expect("graphene-gen-ts should run")
}

#[test]
fn cli_generates_typescript_deterministically() {
    let output_dir = unique_output_dir("cli");
    let first = run_generate(&output_dir);
    assert!(
        first.status.success(),
        "generate failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );

    let generated_path = output_dir.join("typescript/index.ts");
    assert!(generated_path.is_file(), "missing TypeScript output");
    let first_source = fs::read_to_string(&generated_path).expect("TypeScript should read");
    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be UTF-8");
    assert!(first_stdout.contains("generated TypeScript target into"));
    assert!(first_stdout.contains("wrote typescript/index.ts"));

    let second = run_generate(&output_dir);
    assert!(
        second.status.success(),
        "second generate failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        first_stdout,
        String::from_utf8(second.stdout).expect("second stdout should be UTF-8")
    );
    assert_eq!(
        first_source,
        fs::read_to_string(&generated_path).expect("TypeScript should read after rerun")
    );

    fs::remove_dir_all(output_dir).ok();
}

#[test]
fn generated_sample_matches_committed_fixture() {
    let expected_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/generated-samples/typescript/index.ts");
    let output_dir = unique_output_dir("sample");
    let output = run_generate(&output_dir);
    assert!(
        output.status.success(),
        "generate failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = fs::read_to_string(output_dir.join("typescript/index.ts"))
        .expect("generated TypeScript should read");
    let expected = fs::read_to_string(expected_path).expect("expected TypeScript should read");
    assert_eq!(actual, expected);
    assert!(actual.contains("export const rpcMethods"));
    assert!(actual.contains("result: 'DynamicGlobalProperties'"));

    fs::remove_dir_all(output_dir).ok();
}
