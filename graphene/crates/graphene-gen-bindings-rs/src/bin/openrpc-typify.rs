use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;

use serde_json::{Map, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))?;
    let source = fs::read_to_string(&args.schema)
        .map_err(|error| format!("failed to read {}: {error}", args.schema.display()))?;
    let mut schema: Value = serde_json::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", args.schema.display()))?;

    normalize_openrpc_components(&mut schema)?;

    let strip_names = if let Some(path) = &args.strip_names {
        fs::read_to_string(path)?
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    } else {
        BTreeSet::new()
    };

    let generated = graphene_gen_bindings_rs::typify_schema_value(&schema, &strip_names)?;
    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out, generated)?;
    println!("wrote {}", args.out.display());
    Ok(())
}

#[derive(Debug)]
struct Args {
    schema: PathBuf,
    out: PathBuf,
    strip_names: Option<PathBuf>,
}

impl Args {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut schema = None;
        let mut out = None;
        let mut strip_names = None;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--schema" => {
                    schema = Some(PathBuf::from(
                        args.next().ok_or("--schema requires a path")?,
                    ))
                }
                "--out" => out = Some(PathBuf::from(args.next().ok_or("--out requires a path")?)),
                "--strip-names" => {
                    strip_names = Some(PathBuf::from(
                        args.next().ok_or("--strip-names requires a path")?,
                    ));
                }
                "-h" | "--help" => return Err(usage().into()),
                other => return Err(format!("unknown argument: {other}\n{}", usage()).into()),
            }
        }
        Ok(Self {
            schema: schema.ok_or("missing --schema")?,
            out: out.ok_or("missing --out")?,
            strip_names,
        })
    }
}

fn usage() -> &'static str {
    "usage: openrpc-typify-rs --schema schema-or-openrpc.json --out types.rs [--strip-names names.txt]"
}

fn normalize_openrpc_components(schema: &mut Value) -> Result<(), Box<dyn std::error::Error>> {
    let Some(components) = schema.pointer("/components/schemas").cloned() else {
        return Ok(());
    };
    let Some(schemas) = components.as_object() else {
        return Err("components.schemas must be an object".into());
    };
    let defs = schemas
        .iter()
        .map(|(name, schema)| (name.clone(), rewrite_refs_to_defs(schema)))
        .collect::<Map<_, _>>();
    let mut doc = Map::new();
    doc.insert(
        "$schema".to_owned(),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_owned()),
    );
    doc.insert(
        "title".to_owned(),
        Value::String("openrpc_types_root".to_owned()),
    );
    doc.insert("$defs".to_owned(), Value::Object(defs));
    *schema = Value::Object(doc);
    Ok(())
}

fn rewrite_refs_to_defs(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, item)| {
                    if key == "$ref" {
                        if let Some(reference) = item.as_str() {
                            if let Some(name) = reference.strip_prefix("#/components/schemas/") {
                                return (key.clone(), Value::String(format!("#/$defs/{name}")));
                            }
                        }
                    }
                    (key.clone(), rewrite_refs_to_defs(item))
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(rewrite_refs_to_defs).collect()),
        _ => value.clone(),
    }
}
