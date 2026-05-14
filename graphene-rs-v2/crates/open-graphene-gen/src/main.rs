use std::env;
use std::fs;
use std::path::Path;

use open_graphene_gen::model::OpenGrapheneDocument;
use open_graphene_gen::validation::validate_document;
use schemars::schema_for;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("validate") => {
            let path = args
                .next()
                .ok_or("usage: open-graphene-gen validate <path>")?;
            validate_path(Path::new(&path))
        }
        Some("schema") => {
            let schema = schema_for!(OpenGrapheneDocument);
            println!("{}", serde_json::to_string_pretty(&schema)?);
            Ok(())
        }
        _ => Err("usage: open-graphene-gen <validate|schema> [path]".into()),
    }
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
