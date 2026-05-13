use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    match args.next().as_deref().and_then(|arg| arg.to_str()) {
        Some("generate") => {
            let config = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("chains/swaplock.toml"));
            if let Some(extra) = args.next() {
                return Err(format!("unexpected argument: {}", extra.to_string_lossy()).into());
            }
            generate(&config)
        }
        Some("audit") => {
            let config = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("chains/swaplock.toml"));
            if let Some(extra) = args.next() {
                return Err(format!("unexpected argument: {}", extra.to_string_lossy()).into());
            }
            audit(&config)
        }
        _ => Err("usage: graphene-codegen <generate|audit> [config.toml]".into()),
    }
}

#[derive(Debug)]
struct GenerationConfig {
    chain_name: String,
    core_root: PathBuf,
    api_header: String,
    api_qualified_name: String,
    excluded_methods: Vec<String>,
    header_roots: Vec<String>,
    spec_output: PathBuf,
    schema_output: PathBuf,
    variants_rs: PathBuf,
    variants_names: PathBuf,
    types_rs: PathBuf,
    rpc_rs: PathBuf,
    schema_title: String,
}

fn generate(config_path: &Path) -> Result<(), Box<dyn Error>> {
    let workspace_root = find_workspace_root(env::current_dir()?)?;
    let config = load_config(&workspace_root, config_path)?;
    let openrpc_dir = workspace_root.join("crates/graphene-codegen/openrpc");

    run(Command::new(openrpc_dir.join("gen_wallet_openrpc.sh"))
        .arg("--chain-name")
        .arg(&config.chain_name)
        .arg("--core-root")
        .arg(&config.core_root)
        .arg("--api-header")
        .arg(&config.api_header)
        .arg("--api-qualified-name")
        .arg(&config.api_qualified_name)
        .args(
            config
                .excluded_methods
                .iter()
                .flat_map(|method| [OsString::from("--exclude-method"), OsString::from(method)]),
        )
        .arg("--output")
        .arg(&config.spec_output)
        .args(
            config
                .header_roots
                .iter()
                .flat_map(|root| [OsString::from("--header-root"), OsString::from(root)]),
        ))?;

    run(Command::new(openrpc_dir.join("extract_typify_schema.py"))
        .arg("--spec")
        .arg(&config.spec_output)
        .arg("--output")
        .arg(&config.schema_output)
        .arg("--title")
        .arg(&config.schema_title))?;

    run(Command::new(openrpc_dir.join("gen_rust_variants.py"))
        .arg("--spec")
        .arg(&config.spec_output)
        .arg("--out-rs")
        .arg(&config.variants_rs)
        .arg("--out-names")
        .arg(&config.variants_names)
        .arg("--source-label")
        .arg(&config.spec_output))?;

    run(Command::new(openrpc_dir.join("gen_rust_rpc.py"))
        .arg("--spec")
        .arg(&config.spec_output)
        .arg("--out-rs")
        .arg(&config.rpc_rs)
        .arg("--source-label")
        .arg(&config.spec_output))?;

    run(Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(workspace_root.join("Cargo.toml"))
        .arg("-p")
        .arg("graphene-codegen")
        .arg("--bin")
        .arg("openrpc-typify")
        .arg("--")
        .arg("--schema")
        .arg(&config.schema_output)
        .arg("--out")
        .arg(&config.types_rs)
        .arg("--strip-names")
        .arg(&config.variants_names))?;

    Ok(())
}

fn audit(config_path: &Path) -> Result<(), Box<dyn Error>> {
    let workspace_root = find_workspace_root(env::current_dir()?)?;
    let config = load_config(&workspace_root, config_path)?;

    let spec_source = fs::read_to_string(&config.spec_output).map_err(|error| {
        format!(
            "failed to read OpenRPC spec {}: {error}",
            config.spec_output.display()
        )
    })?;
    let spec: JsonValue = serde_json::from_str(&spec_source).map_err(|error| {
        format!(
            "failed to parse OpenRPC spec {}: {error}",
            config.spec_output.display()
        )
    })?;

    let schemas = spec
        .pointer("/components/schemas")
        .and_then(JsonValue::as_object)
        .ok_or("OpenRPC spec is missing components.schemas object")?;
    let methods = spec
        .get("methods")
        .and_then(JsonValue::as_array)
        .ok_or("OpenRPC spec is missing methods array")?;

    let types_rs = fs::read_to_string(&config.types_rs)?;
    let variants_rs = fs::read_to_string(&config.variants_rs)?;
    let variants_names_source = fs::read_to_string(&config.variants_names)?;
    let rpc_rs = fs::read_to_string(&config.rpc_rs)?;
    let variant_names = variants_names_source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let static_variants = schemas
        .iter()
        .filter_map(|(name, schema)| is_static_variant_schema(schema).then_some(name.as_str()))
        .collect::<Vec<_>>();

    let external_schema_types = schemas
        .iter()
        .filter(|(_name, schema)| has_x_rust_type(schema))
        .count();

    let missing_schema_types = schemas
        .iter()
        .filter_map(|(name, schema)| {
            if has_x_rust_type(schema) {
                return None;
            }
            let rust_name = snake_to_pascal(name);
            (!rust_type_exists(&rust_name, &types_rs, &variants_rs)).then_some((name, rust_name))
        })
        .collect::<Vec<_>>();

    let missing_rpc_params = methods
        .iter()
        .filter_map(|method| {
            let name = method.get("name")?.as_str()?;
            let struct_name = format!("{}Params", snake_to_pascal(name));
            (!rpc_rs.contains(&format!("pub struct {struct_name}"))
                || !rpc_rs.contains(&format!("impl OpenRpcParams for {struct_name}")))
            .then_some((name.to_owned(), struct_name))
        })
        .collect::<Vec<_>>();

    let missing_variant_enums = static_variants
        .iter()
        .filter_map(|name| {
            let rust_name = snake_to_pascal(name);
            (!variants_rs.contains(&format!("pub enum {rust_name}"))).then_some((*name, rust_name))
        })
        .collect::<Vec<_>>();

    let missing_variant_names = static_variants
        .iter()
        .filter_map(|name| {
            let rust_name = snake_to_pascal(name);
            (!variant_names
                .iter()
                .any(|variant_name| *variant_name == rust_name))
            .then_some((*name, rust_name))
        })
        .collect::<Vec<_>>();

    let todo_placeholders = count_todo_descriptions(&spec);
    let rpc_value_fields = rpc_rs
        .lines()
        .filter(|line| line.trim_start().starts_with("pub ") && line.contains("serde_json::Value"))
        .count();
    let rpc_value_responses = rpc_rs
        .lines()
        .filter(|line| line.trim() == "type Response = serde_json::Value;")
        .count();

    let mut failures = Vec::new();
    if !rpc_rs.contains("use graphene_rpc::OpenRpcParams;") {
        failures.push("rpc.rs does not import graphene_rpc::OpenRpcParams".to_owned());
    }
    if rpc_rs.contains("pub trait OpenRpcParams") {
        failures.push("rpc.rs still defines a local OpenRpcParams trait".to_owned());
    }
    if !missing_schema_types.is_empty() {
        failures.push(format!(
            "missing schema Rust types: {}",
            format_pairs(&missing_schema_types)
        ));
    }
    if !missing_rpc_params.is_empty() {
        failures.push(format!(
            "missing RPC params/impls: {}",
            format_pairs(&missing_rpc_params)
        ));
    }
    if !missing_variant_enums.is_empty() {
        failures.push(format!(
            "missing static variant enums: {}",
            format_pairs(&missing_variant_enums)
        ));
    }
    if !missing_variant_names.is_empty() {
        failures.push(format!(
            "missing static variant names: {}",
            format_pairs(&missing_variant_names)
        ));
    }

    println!("audit {}", config.chain_name);
    println!("  schemas: {}", schemas.len());
    println!("  methods: {}", methods.len());
    println!("  static variants: {}", static_variants.len());
    println!("  external schema types: {external_schema_types}");
    println!("  missing schema types: {}", missing_schema_types.len());
    println!("  missing RPC params: {}", missing_rpc_params.len());
    println!("  missing variant enums: {}", missing_variant_enums.len());
    println!("  missing variant names: {}", missing_variant_names.len());
    println!("  TODO placeholders: {todo_placeholders}");
    println!("  rpc Value fields: {rpc_value_fields}");
    println!("  rpc Value responses: {rpc_value_responses}");

    if failures.is_empty() {
        if todo_placeholders == 0 {
            println!("  status: pass");
        } else {
            println!("  status: warning");
        }
        Ok(())
    } else {
        println!("  status: fail");
        for failure in &failures {
            println!("  failure: {failure}");
        }
        Err(format!("audit failed for {}", config.chain_name).into())
    }
}

fn load_config(
    workspace_root: &Path,
    config_path: &Path,
) -> Result<GenerationConfig, Box<dyn Error>> {
    let config_path = absolutize(workspace_root, config_path);
    let config_dir = config_path
        .parent()
        .ok_or_else(|| format!("config has no parent directory: {}", config_path.display()))?;
    let config = fs::read_to_string(&config_path)?;
    let config: TomlValue = toml::from_str(&config)?;

    let chain = table(&config, "chain")?;
    let openrpc = table(&config, "openrpc")?;
    let rust = table(&config, "rust")?;

    let chain_name = string(chain, "name")?.to_owned();
    let core_root = resolve(config_dir, string(chain, "core_root")?);
    let api_header = openrpc
        .get("api_header")
        .and_then(TomlValue::as_str)
        .or_else(|| openrpc.get("wallet_header").and_then(TomlValue::as_str))
        .unwrap_or("libraries/wallet/include/graphene/wallet/wallet.hpp")
        .to_owned();
    let api_qualified_name = openrpc
        .get("api_qualified_name")
        .and_then(TomlValue::as_str)
        .unwrap_or("graphene::wallet::wallet_api")
        .to_owned();
    let excluded_methods = openrpc
        .get("excluded_methods")
        .and_then(TomlValue::as_array)
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(ToOwned::to_owned)
                        .ok_or("[openrpc].excluded_methods must contain only strings")
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let header_roots = openrpc
        .get("header_roots")
        .and_then(TomlValue::as_array)
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(ToOwned::to_owned)
                        .ok_or("[openrpc].header_roots must contain only strings")
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let schema_title = rust
        .get("schema_title")
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{chain_name}_types_root"));

    Ok(GenerationConfig {
        chain_name,
        core_root,
        api_header,
        api_qualified_name,
        excluded_methods,
        header_roots,
        spec_output: resolve(config_dir, string(openrpc, "output")?),
        schema_output: resolve(config_dir, string(rust, "schema_output")?),
        variants_rs: resolve(config_dir, string(rust, "variants_rs")?),
        variants_names: resolve(config_dir, string(rust, "variants_names")?),
        types_rs: resolve(config_dir, string(rust, "types_rs")?),
        rpc_rs: resolve(config_dir, string(rust, "rpc_rs")?),
        schema_title,
    })
}

fn is_static_variant_schema(schema: &JsonValue) -> bool {
    let Some(one_of) = schema.get("oneOf").and_then(JsonValue::as_array) else {
        return false;
    };
    !one_of.is_empty()
        && one_of.iter().all(|alternative| {
            alternative.get("type").and_then(JsonValue::as_str) == Some("array")
                && alternative
                    .get("prefixItems")
                    .and_then(JsonValue::as_array)
                    .is_some_and(|items| items.len() == 2 && items[0].get("const").is_some())
        })
}

fn has_x_rust_type(schema: &JsonValue) -> bool {
    schema
        .get("x-rust-type")
        .and_then(JsonValue::as_object)
        .is_some()
}

fn rust_type_exists(rust_name: &str, types_rs: &str, variants_rs: &str) -> bool {
    ["pub struct", "pub enum", "pub type"]
        .iter()
        .any(|prefix| types_rs.contains(&format!("{prefix} {rust_name}")))
        || variants_rs.contains(&format!("pub enum {rust_name}"))
}

fn snake_to_pascal(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}

fn count_todo_descriptions(value: &JsonValue) -> usize {
    match value {
        JsonValue::Object(object) => {
            let here = object
                .get("description")
                .and_then(JsonValue::as_str)
                .is_some_and(|description| description.contains("TODO"))
                as usize;
            here + object.values().map(count_todo_descriptions).sum::<usize>()
        }
        JsonValue::Array(values) => values.iter().map(count_todo_descriptions).sum(),
        _ => 0,
    }
}

fn format_pairs<T: std::fmt::Display, U: std::fmt::Display>(pairs: &[(T, U)]) -> String {
    pairs
        .iter()
        .take(10)
        .map(|(left, right)| format!("{left}->{right}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn table<'a>(
    config: &'a TomlValue,
    name: &str,
) -> Result<&'a toml::map::Map<String, TomlValue>, Box<dyn Error>> {
    config
        .get(name)
        .and_then(TomlValue::as_table)
        .ok_or_else(|| format!("missing [{name}] table").into())
}

fn string<'a>(
    table: &'a toml::map::Map<String, TomlValue>,
    key: &str,
) -> Result<&'a str, Box<dyn Error>> {
    table
        .get(key)
        .and_then(TomlValue::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing string key: {key}").into())
}

fn find_workspace_root(start: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    for directory in start.ancestors() {
        let manifest = directory.join("Cargo.toml");
        if manifest.exists() && fs::read_to_string(&manifest)?.contains("[workspace]") {
            return Ok(directory.to_path_buf());
        }
    }
    Err(format!("could not find workspace root above {}", start.display()).into())
}

fn absolutize(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        normalize(path)
    } else {
        normalize(&base.join(path))
    }
}

fn resolve(base: &Path, value: &str) -> PathBuf {
    absolutize(base, Path::new(value))
}

fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn run(command: &mut Command) -> Result<(), Box<dyn Error>> {
    println!("$ {}", format_command(command));
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "command failed with status {status}: {}",
            format_command(command)
        )
        .into())
    }
}

fn format_command(command: &Command) -> String {
    let mut parts = vec![command.get_program().to_string_lossy().into_owned()];
    parts.extend(
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned()),
    );
    parts.join(" ")
}
