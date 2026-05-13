use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
struct Args {
    schema: PathBuf,
    out: PathBuf,
    strip_names: Option<PathBuf>,
    preview: Option<PathBuf>,
}

impl Args {
    fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut schema = None;
        let mut out = None;
        let mut strip_names = None;
        let mut preview = None;

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--schema" => schema = Some(next_path(&mut arguments, "--schema")?),
                "--out" => out = Some(next_path(&mut arguments, "--out")?),
                "--strip-names" => strip_names = Some(next_path(&mut arguments, "--strip-names")?),
                "--preview" => preview = Some(next_path(&mut arguments, "--preview")?),
                "-h" | "--help" => return Err(usage()),
                _ => return Err(format!("unknown argument: {argument}\n\n{}", usage())),
            }
        }

        Ok(Self {
            schema: schema.ok_or_else(|| "missing --schema".to_owned())?,
            out: out.ok_or_else(|| "missing --out".to_owned())?,
            strip_names,
            preview,
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
    "usage: openrpc-typify --schema schema.json --out types.rs [--strip-names variants.names] [--preview preview.rs]".to_owned()
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

    let schema_source = fs::read_to_string(&args.schema)
        .map_err(|error| format!("failed to read schema {}: {error}", args.schema.display()))?;
    let parsed: schemars::schema::RootSchema = serde_json::from_str(&schema_source)
        .map_err(|error| format!("failed to parse schema {}: {error}", args.schema.display()))?;

    let mut settings = typify::TypeSpaceSettings::default();
    settings.with_struct_builder(true);
    let mut typespace = typify::TypeSpace::new(&settings);
    typespace.add_root_schema(parsed).map_err(|error| {
        format!(
            "typify failed to ingest schema {}: {error}",
            args.schema.display()
        )
    })?;

    let tokens = typespace.to_stream();
    let formatted = match syn::parse2::<syn::File>(tokens.clone()) {
        Ok(file) => prettyplease::unparse(&file),
        Err(_) => tokens.to_string(),
    };

    let final_source = if let Some(strip_names) = &args.strip_names {
        let names = read_strip_names(strip_names)?;
        strip_named_items(&formatted, &names)
    } else {
        formatted
    };

    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out, &final_source)
        .map_err(|error| format!("failed to write {}: {error}", args.out.display()))?;

    if let Some(preview) = &args.preview {
        if let Some(parent) = preview.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(preview, &final_source)
            .map_err(|error| format!("failed to write {}: {error}", preview.display()))?;
    }

    let line_count = final_source.lines().count();
    println!(
        "Wrote {} ({} lines{})",
        args.out.display(),
        line_count,
        args.strip_names
            .as_ref()
            .map(|path| format!(", stripped names from {}", path.display()))
            .unwrap_or_default()
    );

    Ok(())
}

fn read_strip_names(path: &PathBuf) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read strip names {}: {error}", path.display()))?;
    Ok(source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn strip_named_items(source: &str, names: &HashSet<String>) -> String {
    let mut file: syn::File = match syn::parse_str(source) {
        Ok(file) => file,
        Err(_) => return source.to_owned(),
    };

    file.items.retain(|item| keep_item(item, names));
    for item in &mut file.items {
        if let syn::Item::Mod(module) = item {
            if let Some((_brace, items)) = &mut module.content {
                items.retain(|item| keep_item(item, names));
            }
        }
    }

    prettyplease::unparse(&file)
}

fn keep_item(item: &syn::Item, names: &HashSet<String>) -> bool {
    match item {
        syn::Item::Enum(item) => !names.contains(&item.ident.to_string()),
        syn::Item::Struct(item) => !names.contains(&item.ident.to_string()),
        syn::Item::Impl(item) => {
            !(type_mentions_name(&item.self_ty, names)
                || item
                    .trait_
                    .as_ref()
                    .is_some_and(|(_bang, path, _for)| path_mentions_name(path, names)))
        }
        _ => true,
    }
}

fn type_mentions_name(ty: &syn::Type, names: &HashSet<String>) -> bool {
    match ty {
        syn::Type::Path(path) => path_mentions_name(&path.path, names),
        syn::Type::Reference(reference) => type_mentions_name(&reference.elem, names),
        syn::Type::Array(array) => type_mentions_name(&array.elem, names),
        syn::Type::Tuple(tuple) => tuple.elems.iter().any(|ty| type_mentions_name(ty, names)),
        syn::Type::Paren(paren) => type_mentions_name(&paren.elem, names),
        syn::Type::Group(group) => type_mentions_name(&group.elem, names),
        _ => false,
    }
}

fn path_mentions_name(path: &syn::Path, names: &HashSet<String>) -> bool {
    path.segments.iter().any(|segment| {
        names.contains(&segment.ident.to_string())
            || match &segment.arguments {
                syn::PathArguments::AngleBracketed(arguments) => arguments.args.iter().any(|arg| {
                    matches!(arg, syn::GenericArgument::Type(ty) if type_mentions_name(ty, names))
                }),
                syn::PathArguments::Parenthesized(arguments) => {
                    arguments.inputs.iter().any(|ty| type_mentions_name(ty, names))
                        || matches!(&arguments.output, syn::ReturnType::Type(_, ty) if type_mentions_name(ty, names))
                }
                syn::PathArguments::None => false,
            }
    })
}

#[cfg(test)]
mod tests {
    use super::{strip_named_items, Args};
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[test]
    fn parses_required_arguments() {
        let args = Args::parse(
            [
                "--schema",
                "schema.json",
                "--out",
                "types.rs",
                "--strip-names",
                "variants.names",
                "--preview",
                "preview.rs",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();

        assert_eq!(
            args,
            Args {
                schema: PathBuf::from("schema.json"),
                out: PathBuf::from("types.rs"),
                strip_names: Some(PathBuf::from("variants.names")),
                preview: Some(PathBuf::from("preview.rs")),
            }
        );
    }

    #[test]
    fn strips_named_enums_structs_and_impls() {
        let source = r#"
            pub enum Operation { Variant }
            impl Operation { pub fn method(&self) {} }
            impl serde::Serialize for Operation { fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error> where S: serde::Serializer { todo!() } }
            impl std::convert::From<Operation> for [serde_json::Value; 2] { fn from(value: Operation) -> Self { todo!() } }
            pub struct KeepMe;
        "#;
        let names = HashSet::from(["Operation".to_owned()]);
        let stripped = strip_named_items(source, &names);

        assert!(!stripped.contains("enum Operation"));
        assert!(!stripped.contains("impl Operation"));
        assert!(!stripped.contains("for Operation"));
        assert!(stripped.contains("struct KeepMe"));
    }
}
