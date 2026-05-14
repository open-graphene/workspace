use std::env;
use std::fs;
use std::path::Path;

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
            let path = args
                .next()
                .ok_or("usage: open-graphene-gen inspect-ir <opengraphene-path> [--openrpc <openrpc-path>]")?;
            let openrpc_path = parse_inspect_ir_args(args)?;
            inspect_ir_path(Path::new(&path), openrpc_path.as_deref().map(Path::new))
        }
        _ => Err("usage: open-graphene-gen <validate|schema|inspect-ir> [args]".into()),
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
    let contents = fs::read_to_string(path)?;
    let document: OpenGrapheneDocument = serde_json::from_str(&contents)?;

    let ir = if let Some(openrpc_path) = openrpc_path {
        let openrpc_contents = fs::read_to_string(openrpc_path)?;
        let openrpc = OpenRpcDocument::parse_json(&openrpc_contents)?;
        lower_document_with_openrpc_to_ir(&document, &openrpc)?
    } else {
        lower_document_to_ir(&document)?
    };

    println!("{}", serde_json::to_string_pretty(&ir)?);
    Ok(())
}
