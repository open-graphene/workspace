use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use graphene_spec_gen::conformance::{build_swaplock_transfer_fixture, ConformanceFixtureError};
use graphene_spec_gen::emit::dart::emit_dart;
use graphene_spec_gen::emit::rust::emit_rust;
use graphene_spec_gen::emit::typescript::emit_typescript;
use graphene_spec_gen::emit::{write_generated_files, GeneratedFile};
use graphene_spec_gen::ir::IrDocument;
use graphene_spec_gen::lower::{lower_document_to_ir, lower_document_with_openrpc_to_ir};
use graphene_spec_gen::model::OpenGrapheneDocument;
use graphene_spec_gen::openrpc::OpenRpcDocument;
use graphene_spec_gen::validation::validate_document_report;
use schemars::schema_for;

const BUILTIN_OPENGRAPHENE_SPECS: &[(&str, &str)] = &[
    (
        "swaplock.opengraphene.json",
        include_str!("../../fixtures/swaplock.opengraphene.json"),
    ),
    (
        "acta.opengraphene.json",
        include_str!("../../fixtures/acta.opengraphene.json"),
    ),
];

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("validate") => {
            let path = args
                .next()
                .ok_or("usage: graphene-spec-gen validate <path>")?;
            reject_extra_args(args)?;
            validate_path(Path::new(&path))
        }
        Some("schema") => {
            reject_extra_args(args)?;
            let schema = schema_for!(OpenGrapheneDocument);
            println!("{}", serde_json::to_string_pretty(&schema)?);
            Ok(())
        }
        Some("inspect-ir") => {
            let path = args.next().ok_or(
                "usage: graphene-spec-gen inspect-ir <opengraphene-path> [--openrpc <openrpc-path>]",
            )?;
            let openrpc_path = parse_inspect_ir_args(args)?;
            inspect_ir_path(Path::new(&path), openrpc_path.as_deref().map(Path::new))
        }
        Some("generate") => {
            let path = args.next().ok_or(
                "usage: graphene-spec-gen generate <opengraphene-path> [--openrpc <openrpc-path>] --target <typescript|dart|rust|all> --out <dir>",
            )?;
            let options = parse_generate_args(args)?;
            generate_path(Path::new(&path), &options)
        }
        Some("conformance-fixtures") => {
            let output_dir = parse_conformance_fixtures_args(args)?;
            write_conformance_fixtures(&output_dir)
        }
        Some("opengraphene-specs") => {
            let output_dir = parse_opengraphene_specs_args(args)?;
            write_opengraphene_specs(&output_dir)
        }
        _ => Err(
            "usage: graphene-spec-gen <validate|schema|inspect-ir|generate|conformance-fixtures|opengraphene-specs> [args]"
                .into(),
        ),
    }
}

fn reject_extra_args(
    mut args: impl Iterator<Item = String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(arg) = args.next() {
        Err(format!("unexpected argument: {arg}").into())
    } else {
        Ok(())
    }
}

fn parse_inspect_ir_args(
    mut args: impl Iterator<Item = String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut openrpc_path = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--openrpc" => {
                if openrpc_path.is_some() {
                    return Err("--openrpc may only be provided once".into());
                }
                openrpc_path = Some(args.next().ok_or("--openrpc requires a path")?);
            }
            _ => return Err(format!("unexpected inspect-ir argument: {arg}").into()),
        }
    }

    Ok(openrpc_path)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GenerateTarget {
    TypeScript,
    Dart,
    Rust,
    All,
}

impl GenerateTarget {
    fn parse(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match value {
            "typescript" => Ok(Self::TypeScript),
            "dart" => Ok(Self::Dart),
            "rust" => Ok(Self::Rust),
            "all" => Ok(Self::All),
            _ => Err(format!(
                "unsupported generate target '{value}'; expected typescript, dart, rust, or all"
            )
            .into()),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::TypeScript => "typescript",
            Self::Dart => "dart",
            Self::Rust => "rust",
            Self::All => "all",
        }
    }

    fn includes_typescript(self) -> bool {
        matches!(self, Self::TypeScript | Self::All)
    }

    fn includes_dart(self) -> bool {
        matches!(self, Self::Dart | Self::All)
    }

    fn includes_rust(self) -> bool {
        matches!(self, Self::Rust | Self::All)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GenerateOptions {
    openrpc_path: Option<PathBuf>,
    target: GenerateTarget,
    output_dir: PathBuf,
}

fn parse_generate_args(
    mut args: impl Iterator<Item = String>,
) -> Result<GenerateOptions, Box<dyn std::error::Error>> {
    let mut openrpc_path = None;
    let mut target = None;
    let mut output_dir = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--openrpc" => {
                if openrpc_path.is_some() {
                    return Err("--openrpc may only be provided once".into());
                }
                openrpc_path = Some(PathBuf::from(
                    args.next().ok_or("--openrpc requires a path")?,
                ));
            }
            "--target" => {
                if target.is_some() {
                    return Err("--target may only be provided once".into());
                }
                target =
                    Some(GenerateTarget::parse(&args.next().ok_or(
                        "--target requires typescript, dart, rust, or all",
                    )?)?);
            }
            "--out" => {
                if output_dir.is_some() {
                    return Err("--out may only be provided once".into());
                }
                output_dir = Some(PathBuf::from(
                    args.next().ok_or("--out requires a directory")?,
                ));
            }
            _ => return Err(format!("unexpected generate argument: {arg}").into()),
        }
    }

    Ok(GenerateOptions {
        openrpc_path,
        target: target.ok_or("generate requires --target <typescript|dart|rust|all>")?,
        output_dir: output_dir.ok_or("generate requires --out <dir>")?,
    })
}

fn parse_conformance_fixtures_args(
    mut args: impl Iterator<Item = String>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    parse_out_dir_args(&mut args, "conformance-fixtures")
}

fn parse_opengraphene_specs_args(
    mut args: impl Iterator<Item = String>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    parse_out_dir_args(&mut args, "opengraphene-specs")
}

fn parse_out_dir_args(
    args: &mut impl Iterator<Item = String>,
    command: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut output_dir = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => {
                if output_dir.is_some() {
                    return Err("--out may only be provided once".into());
                }
                output_dir = Some(PathBuf::from(
                    args.next().ok_or("--out requires a directory")?,
                ));
            }
            _ => return Err(format!("unexpected {command} argument: {arg}").into()),
        }
    }

    output_dir.ok_or(format!("{command} requires --out <dir>").into())
}

fn write_opengraphene_specs(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for (name, contents) in BUILTIN_OPENGRAPHENE_SPECS {
        let document: OpenGrapheneDocument = serde_json::from_str(contents)?;
        let report = validate_document_report(&document);
        if !report.is_valid() {
            return Err(format!(
                "built-in {name} OpenGraphene spec is invalid: {} validation error(s)",
                report.errors.len()
            )
            .into());
        }
        let json = serde_json::to_string_pretty(&document)?;
        files.push(GeneratedFile::new(*name, format!("{json}\n"))?);
    }

    let written = write_generated_files(output_dir, &files)?;

    println!("generated OpenGraphene specs into {}", output_dir.display());
    for file in written {
        println!("  wrote {} ({} bytes)", file.path.display(), file.bytes);
    }

    Ok(())
}

fn write_conformance_fixtures(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = build_swaplock_transfer_fixture().map_err(describe_conformance_error)?;
    let fixture_json = serde_json::to_string_pretty(&fixture)
        .map_err(|error| format!("conformance fixture json_envelope failed: {error}"))?;
    let files = vec![GeneratedFile::new(
        "transfer.json",
        format!("{fixture_json}\n"),
    )?];
    let written = write_generated_files(output_dir, &files)?;

    println!(
        "generated conformance fixtures into {}",
        output_dir.display()
    );
    for file in written {
        println!("  wrote {} ({} bytes)", file.path.display(), file.bytes);
    }

    Ok(())
}

fn describe_conformance_error(error: ConformanceFixtureError) -> String {
    format!(
        "conformance fixture {:?} failed: {}",
        error.phase, error.message
    )
}

fn validate_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let document: OpenGrapheneDocument = serde_json::from_str(&contents)?;
    let report = validate_document_report(&document);
    if report.is_valid() {
        for warning in &report.warnings {
            eprintln!("{}: warning: {warning}", path.display());
        }
        println!("{}: valid OpenGraphene contract", path.display());
        Ok(())
    } else {
        for error in &report.errors {
            eprintln!("{}: {error}", path.display());
        }
        Err(format!("{} validation error(s)", report.errors.len()).into())
    }
}

fn inspect_ir_path(
    path: &Path,
    openrpc_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ir = lower_ir_path(path, openrpc_path)?;

    println!("{}", serde_json::to_string_pretty(&ir)?);
    Ok(())
}

fn generate_path(path: &Path, options: &GenerateOptions) -> Result<(), Box<dyn std::error::Error>> {
    let ir = lower_ir_path(path, options.openrpc_path.as_deref())?;
    let files = emit_target(&ir, options.target)?;
    let written = write_generated_files(&options.output_dir, &files)?;

    println!(
        "generated target '{}' into {}",
        options.target.label(),
        options.output_dir.display()
    );
    for file in written {
        println!("  wrote {} ({} bytes)", file.path.display(), file.bytes);
    }

    Ok(())
}

fn lower_ir_path(
    path: &Path,
    openrpc_path: Option<&Path>,
) -> Result<IrDocument, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let document: OpenGrapheneDocument = serde_json::from_str(&contents)?;

    if let Some(openrpc_path) = openrpc_path {
        let openrpc_contents = fs::read_to_string(openrpc_path)?;
        let openrpc = OpenRpcDocument::parse_json(&openrpc_contents)?;
        Ok(lower_document_with_openrpc_to_ir(&document, &openrpc)?)
    } else {
        Ok(lower_document_to_ir(&document)?)
    }
}

fn emit_target(
    ir: &IrDocument,
    target: GenerateTarget,
) -> Result<Vec<GeneratedFile>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    if target.includes_typescript() {
        files.extend(emit_typescript(ir)?);
    }
    if target.includes_dart() {
        files.extend(emit_dart(ir)?);
    }
    if target.includes_rust() {
        files.extend(emit_rust(ir)?);
    }

    Ok(files)
}
