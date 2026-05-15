use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use graphene_gen_dart::emit_dart;
use graphene_spec_gen::emit::write_generated_files;
use graphene_spec_gen::lower::{lower_document_to_ir, lower_document_with_openrpc_to_ir};
use graphene_spec_gen::model::OpenGrapheneDocument;
use graphene_spec_gen::openrpc::OpenRpcDocument;

#[derive(Debug, Eq, PartialEq)]
struct Args {
    opengraphene_path: PathBuf,
    openrpc_path: Option<PathBuf>,
    out: PathBuf,
}

impl Args {
    fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        match arguments.next().as_deref() {
            Some("generate") => {}
            Some("-h" | "--help") => return Err(usage()),
            Some(command) => return Err(format!("unknown command: {command}\n\n{}", usage())),
            None => return Err(usage()),
        }

        let opengraphene_path = arguments.next().map(PathBuf::from).ok_or_else(usage)?;
        let mut openrpc_path = None;
        let mut out = None;

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--openrpc" => {
                    if openrpc_path.is_some() {
                        return Err("--openrpc may only be provided once".to_owned());
                    }
                    openrpc_path = Some(next_path(&mut arguments, "--openrpc")?);
                }
                "--out" => {
                    if out.is_some() {
                        return Err("--out may only be provided once".to_owned());
                    }
                    out = Some(next_path(&mut arguments, "--out")?);
                }
                _ => return Err(format!("unexpected generate argument: {argument}")),
            }
        }

        Ok(Self {
            opengraphene_path,
            openrpc_path,
            out: out.ok_or_else(|| "generate requires --out <dir>".to_owned())?,
        })
    }
}

fn next_path(
    arguments: &mut impl Iterator<Item = String>,
    flag: &'static str,
) -> Result<PathBuf, String> {
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{flag} requires a path"))
}

fn usage() -> String {
    "usage: graphene-gen-dart generate <opengraphene-path> [--openrpc <openrpc-path>] --out <dir>"
        .to_owned()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1)).map_err(|error| {
        let kind = if error.starts_with("usage:") {
            std::io::ErrorKind::Other
        } else {
            std::io::ErrorKind::InvalidInput
        };
        std::io::Error::new(kind, error)
    })?;

    let ir = lower_ir_path(&args.opengraphene_path, args.openrpc_path.as_deref())?;
    let files = emit_dart(&ir)?;
    let written = write_generated_files(&args.out, &files)?;

    println!("generated Dart target into {}", args.out.display());
    for file in written {
        println!("  wrote {} ({} bytes)", file.path.display(), file.bytes);
    }

    Ok(())
}

fn lower_ir_path(
    path: &Path,
    openrpc_path: Option<&Path>,
) -> Result<graphene_spec_gen::ir::IrDocument, Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {
    use super::Args;
    use std::path::PathBuf;

    #[test]
    fn parses_generate_arguments() {
        let args = Args::parse(
            [
                "generate",
                "specs/swaplock.opengraphene.json",
                "--openrpc",
                "fixtures/swaplock.openrpc.json",
                "--out",
                "out",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();

        assert_eq!(
            args,
            Args {
                opengraphene_path: PathBuf::from("specs/swaplock.opengraphene.json"),
                openrpc_path: Some(PathBuf::from("fixtures/swaplock.openrpc.json")),
                out: PathBuf::from("out"),
            }
        );
    }
}
