use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use open_graphene_gen::emit::dart::emit_dart;
use open_graphene_gen::emit::typescript::emit_typescript;
use open_graphene_gen::emit::{write_generated_files, GeneratedFile};
use open_graphene_gen::ir::IrDocument;
use open_graphene_gen::lower::{lower_document_to_ir, lower_document_with_openrpc_to_ir};
use open_graphene_gen::model::OpenGrapheneDocument;
use open_graphene_gen::openrpc::OpenRpcDocument;
use open_graphene_gen::validation::validate_document;
use schemars::schema_for;

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
                .ok_or("usage: open-graphene-gen validate <path>")?;
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
                "usage: open-graphene-gen inspect-ir <opengraphene-path> [--openrpc <openrpc-path>]",
            )?;
            let openrpc_path = parse_inspect_ir_args(args)?;
            inspect_ir_path(Path::new(&path), openrpc_path.as_deref().map(Path::new))
        }
        Some("generate") => {
            let path = args.next().ok_or(
                "usage: open-graphene-gen generate <opengraphene-path> [--openrpc <openrpc-path>] --target <typescript|dart|all> --out <dir>",
            )?;
            let options = parse_generate_args(args)?;
            generate_path(Path::new(&path), &options)
        }
        _ => Err("usage: open-graphene-gen <validate|schema|inspect-ir|generate> [args]".into()),
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
    All,
}

impl GenerateTarget {
    fn parse(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match value {
            "typescript" => Ok(Self::TypeScript),
            "dart" => Ok(Self::Dart),
            "all" => Ok(Self::All),
            _ => Err(format!(
                "unsupported generate target '{value}'; expected typescript, dart, or all"
            )
            .into()),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::TypeScript => "typescript",
            Self::Dart => "dart",
            Self::All => "all",
        }
    }

    fn includes_typescript(self) -> bool {
        matches!(self, Self::TypeScript | Self::All)
    }

    fn includes_dart(self) -> bool {
        matches!(self, Self::Dart | Self::All)
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
                target = Some(GenerateTarget::parse(
                    &args
                        .next()
                        .ok_or("--target requires typescript, dart, or all")?,
                )?);
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
        target: target.ok_or("generate requires --target <typescript|dart|all>")?,
        output_dir: output_dir.ok_or("generate requires --out <dir>")?,
    })
}

fn validate_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let document: OpenGrapheneDocument = serde_json::from_str(&contents)?;
    match validate_document(&document) {
        Ok(()) => {
            println!("{}: valid OpenGraphene contract", path.display());
            Ok(())
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("{}: {error}", path.display());
            }
            Err(format!("{} validation error(s)", errors.len()).into())
        }
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

    Ok(files)
}
