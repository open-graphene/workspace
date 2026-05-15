use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use graphene_spec_gen::emit::{write_generated_files, GeneratedFile};

fn unique_output_dir(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "graphene-spec-gen-{test_name}-{}-{nanos}",
        std::process::id()
    ))
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
