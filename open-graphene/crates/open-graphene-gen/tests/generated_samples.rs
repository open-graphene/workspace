use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_path(name: &str) -> String {
    format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

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

fn run_generate(output_dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_open-graphene-gen"))
        .args([
            "generate",
            &fixture_path("swaplock.opengraphene.json"),
            "--openrpc",
            &fixture_path("swaplock.openrpc.json"),
            "--target",
            "all",
            "--out",
            output_dir.to_str().expect("temp path should be UTF-8"),
        ])
        .output()
        .expect("generate command should start")
}

fn generated_file_set(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();

    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
        {
            let entry = entry.expect("directory entry should be readable");
            let path = entry.path();
            let file_type = entry.file_type().expect("file type should be readable");
            if file_type.is_dir() {
                pending.push(path);
            } else if file_type.is_file() {
                files.push(
                    path.strip_prefix(root)
                        .expect("generated file should be inside root")
                        .to_path_buf(),
                );
            }
        }
    }

    files.sort();
    files
}

#[test]
fn generated_samples_match_committed_fixture() {
    let expected_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/generated-samples");
    let output_dir = unique_output_dir("generated-samples");

    let output = run_generate(&output_dir);
    assert!(
        output.status.success(),
        "generate failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let expected_files = generated_file_set(&expected_root);
    let actual_files = generated_file_set(&output_dir);
    assert_eq!(
        actual_files, expected_files,
        "generated sample file structure changed"
    );
    assert_eq!(
        actual_files,
        vec![
            PathBuf::from("dart/lib/open_graphene.dart"),
            PathBuf::from("rust/src/lib.rs"),
            PathBuf::from("typescript/index.ts"),
        ],
        "generated sample fixture should stay intentionally small"
    );

    for relative_path in expected_files {
        let expected_path = expected_root.join(&relative_path);
        let actual_path = output_dir.join(&relative_path);
        let expected = fs::read_to_string(&expected_path).unwrap_or_else(|error| {
            panic!("{} should be readable: {error}", expected_path.display())
        });
        let actual = fs::read_to_string(&actual_path).unwrap_or_else(|error| {
            panic!("{} should be readable: {error}", actual_path.display())
        });
        assert_eq!(
            actual, expected,
            "generated sample {} changed; regenerate fixtures/generated-samples intentionally if emitter behavior changed",
            relative_path.display()
        );
    }

    let typescript = fs::read_to_string(expected_root.join("typescript/index.ts"))
        .expect("TypeScript sample should be readable");
    assert!(
        typescript.contains("export const rpcMethods"),
        "TypeScript sample should expose RPC metadata for human review"
    );
    assert!(
        typescript.contains("result: 'DynamicGlobalProperties'"),
        "TypeScript sample should include OpenRPC result enrichment"
    );

    let dart = fs::read_to_string(expected_root.join("dart/lib/open_graphene.dart"))
        .expect("Dart sample should be readable");
    assert!(
        dart.contains("const Map<String, RpcMethodDescriptor> rpcMethods"),
        "Dart sample should expose RPC metadata for human review"
    );
    assert!(
        dart.contains("result: 'DynamicGlobalProperties'"),
        "Dart sample should include OpenRPC result enrichment"
    );

    let rust = fs::read_to_string(expected_root.join("rust/src/lib.rs"))
        .expect("Rust sample should be readable");
    assert!(
        rust.contains("pub const RPC_METHODS"),
        "Rust sample should expose RPC metadata for human review"
    );
    assert!(
        rust.contains("DynamicGlobalProperties"),
        "Rust sample should include OpenRPC result enrichment"
    );

    fs::remove_dir_all(output_dir).ok();
}
