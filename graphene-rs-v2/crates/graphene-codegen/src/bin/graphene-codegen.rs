use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use toml::Value;

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
        _ => Err("usage: graphene-codegen generate [config.toml]".into()),
    }
}

fn generate(config_path: &Path) -> Result<(), Box<dyn Error>> {
    let workspace_root = find_workspace_root(env::current_dir()?)?;
    let config_path = absolutize(&workspace_root, config_path);
    let config_dir = config_path
        .parent()
        .ok_or_else(|| format!("config has no parent directory: {}", config_path.display()))?;
    let config = fs::read_to_string(&config_path)?;
    let config: Value = toml::from_str(&config)?;

    let chain = table(&config, "chain")?;
    let openrpc = table(&config, "openrpc")?;
    let rust = table(&config, "rust")?;

    let chain_name = string(chain, "name")?;
    let core_root = resolve(config_dir, string(chain, "core_root")?);
    let wallet_header = openrpc
        .get("wallet_header")
        .and_then(Value::as_str)
        .unwrap_or("libraries/wallet/include/graphene/wallet/wallet.hpp");
    let header_roots = openrpc
        .get("header_roots")
        .and_then(Value::as_array)
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
        .transpose()?;
    let header_roots = header_roots.unwrap_or_default();

    let spec_output = resolve(config_dir, string(openrpc, "output")?);
    let schema_output = resolve(config_dir, string(rust, "schema_output")?);
    let variants_rs = resolve(config_dir, string(rust, "variants_rs")?);
    let variants_names = resolve(config_dir, string(rust, "variants_names")?);
    let types_rs = resolve(config_dir, string(rust, "types_rs")?);
    let rpc_rs = resolve(config_dir, string(rust, "rpc_rs")?);
    let schema_title = rust
        .get("schema_title")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{chain_name}_types_root"));

    let openrpc_dir = workspace_root.join("crates/graphene-codegen/openrpc");

    run(Command::new(openrpc_dir.join("gen_wallet_openrpc.sh"))
        .arg("--chain-name")
        .arg(chain_name)
        .arg("--core-root")
        .arg(&core_root)
        .arg("--wallet-header")
        .arg(wallet_header)
        .arg("--output")
        .arg(&spec_output)
        .args(
            header_roots
                .iter()
                .flat_map(|root| [OsString::from("--header-root"), OsString::from(root)]),
        ))?;

    run(Command::new(openrpc_dir.join("extract_typify_schema.py"))
        .arg("--spec")
        .arg(&spec_output)
        .arg("--output")
        .arg(&schema_output)
        .arg("--title")
        .arg(schema_title))?;

    run(Command::new(openrpc_dir.join("gen_rust_variants.py"))
        .arg("--spec")
        .arg(&spec_output)
        .arg("--out-rs")
        .arg(&variants_rs)
        .arg("--out-names")
        .arg(&variants_names)
        .arg("--source-label")
        .arg(&spec_output))?;

    run(Command::new(openrpc_dir.join("gen_rust_rpc.py"))
        .arg("--spec")
        .arg(&spec_output)
        .arg("--out-rs")
        .arg(&rpc_rs)
        .arg("--source-label")
        .arg(&spec_output))?;

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
        .arg(&schema_output)
        .arg("--out")
        .arg(&types_rs)
        .arg("--strip-names")
        .arg(&variants_names))?;

    Ok(())
}

fn table<'a>(
    config: &'a Value,
    name: &str,
) -> Result<&'a toml::map::Map<String, Value>, Box<dyn Error>> {
    config
        .get(name)
        .and_then(Value::as_table)
        .ok_or_else(|| format!("missing [{name}] table").into())
}

fn string<'a>(
    table: &'a toml::map::Map<String, Value>,
    key: &str,
) -> Result<&'a str, Box<dyn Error>> {
    table
        .get(key)
        .and_then(Value::as_str)
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
