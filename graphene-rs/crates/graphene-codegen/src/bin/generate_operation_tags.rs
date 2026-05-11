use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use graphene_codegen::{
    load_codegen_config, parse_operation_variants, render_operation_variants_module,
};

struct ChainConfig {
    name: String,
    core_path: String,
    crate_path: PathBuf,
    config_path: PathBuf,
}

struct GeneratedFile {
    path: PathBuf,
    contents: String,
    variant_count: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct Args {
    write: bool,
    config_path: Option<PathBuf>,
}

impl Args {
    fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut write = false;
        let mut config_path = None;
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--write" => write = true,
                "--config" => {
                    let Some(path) = arguments.next() else {
                        return Err("--config requires a path".to_owned());
                    };
                    config_path = Some(PathBuf::from(path));
                }
                _ => return Err(format!("unknown argument: {argument}")),
            }
        }

        Ok(Self { write, config_path })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse(env::args().skip(1))
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let graphene_v2_root = find_graphene_v2_root()?;
    let chains = read_chain_configs(&graphene_v2_root, args.config_path.as_deref())?;

    println!(
        "graphene-codegen operation-tag generation ({})",
        if args.write { "write" } else { "dry-run" }
    );
    println!("root: {}", graphene_v2_root.display());

    let mut total_variants = 0usize;
    for chain in &chains {
        let operations_header_path = operations_header_path(&graphene_v2_root, chain);
        let generated_file = render_chain_file(&graphene_v2_root, chain)?;
        total_variants += generated_file.variant_count;

        println!("{}:", chain.name);
        println!("  config: {}", chain.config_path.display());
        println!("  operations header: {}", operations_header_path.display());
        println!("  variants: {}", generated_file.variant_count);
        println!("  output: {}", generated_file.path.display());

        if args.write {
            if let Some(parent) = generated_file.path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&generated_file.path, generated_file.contents)?;
        }
    }

    if !args.write {
        println!("dry-run only; pass --write to update generated files");
    }
    println!("planned variants: {total_variants}");

    Ok(())
}

fn read_chain_configs(
    graphene_v2_root: &Path,
    config_path: Option<&Path>,
) -> Result<Vec<ChainConfig>, Box<dyn std::error::Error>> {
    let mut config_paths = if let Some(config_path) = config_path {
        vec![resolve_config_path(graphene_v2_root, config_path)]
    } else {
        let crates_path = graphene_v2_root.join("graphene-rs/crates");
        fs::read_dir(&crates_path)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_dir()
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("graphene-chain-"))
            })
            .map(|crate_path| crate_path.join("codegen.toml"))
            .filter(|config_path| config_path.exists())
            .collect::<Vec<_>>()
    };
    config_paths.sort();

    let mut chains = Vec::with_capacity(config_paths.len());
    for config_path in config_paths {
        let config = load_codegen_config(&config_path)?;
        let crate_path = config_path
            .parent()
            .ok_or_else(|| format!("{} has no parent directory", config_path.display()))?
            .to_path_buf();
        chains.push(ChainConfig {
            name: config.chain.name,
            core_path: config.chain.core_path,
            crate_path,
            config_path,
        });
    }

    Ok(chains)
}

fn resolve_config_path(graphene_v2_root: &Path, config_path: &Path) -> PathBuf {
    if config_path.is_absolute() {
        return config_path.to_path_buf();
    }

    graphene_v2_root.join("graphene-rs").join(config_path)
}

fn render_chain_file(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
) -> Result<GeneratedFile, Box<dyn std::error::Error>> {
    let operations_header_path = operations_header_path(graphene_v2_root, chain);
    let source = fs::read_to_string(&operations_header_path).map_err(|error| {
        format!(
            "failed to read operations header {} for config {}: {error}",
            operations_header_path.display(),
            chain.config_path.display()
        )
    })?;
    let variants = parse_operation_variants(&source).map_err(|error| {
        format!(
            "failed to parse operation variants from {} for config {}: {error}",
            operations_header_path.display(),
            chain.config_path.display()
        )
    })?;
    let contents = render_operation_variants_module(&chain.name, &variants);

    Ok(GeneratedFile {
        path: chain.crate_path.join("src/operation_variants.rs"),
        contents,
        variant_count: variants.len(),
    })
}

fn operations_header_path(graphene_v2_root: &Path, chain: &ChainConfig) -> PathBuf {
    graphene_v2_root
        .join(&chain.core_path)
        .join("libraries/protocol/include/graphene/protocol/operations.hpp")
}

fn find_graphene_v2_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    for ancestor in current_dir.ancestors() {
        if is_graphene_v2_root(ancestor) {
            return Ok(ancestor.to_path_buf());
        }

        let nested = ancestor.join("graphene-v2");
        if is_graphene_v2_root(&nested) {
            return Ok(nested);
        }
    }

    Err(format!(
        "could not find graphene-v2 root from {}",
        current_dir.display()
    )
    .into())
}

fn is_graphene_v2_root(path: &Path) -> bool {
    path.join("chains").is_dir() && path.join("graphene-rs").is_dir()
}

#[cfg(test)]
mod tests {
    use super::Args;
    use std::path::PathBuf;

    fn parse(arguments: &[&str]) -> Result<Args, String> {
        Args::parse(arguments.iter().map(|argument| (*argument).to_owned()))
    }

    #[test]
    fn parses_write_and_config_arguments() {
        assert_eq!(
            parse(&[
                "--config",
                "crates/graphene-chain-bitshares/codegen.toml",
                "--write"
            ]),
            Ok(Args {
                write: true,
                config_path: Some(PathBuf::from(
                    "crates/graphene-chain-bitshares/codegen.toml"
                )),
            })
        );
    }

    #[test]
    fn rejects_unknown_argument() {
        assert_eq!(
            parse(&["--wat"]).expect_err("unknown argument should fail"),
            "unknown argument: --wat"
        );
    }

    #[test]
    fn rejects_config_without_path() {
        assert_eq!(
            parse(&["--config"]).expect_err("missing config path should fail"),
            "--config requires a path"
        );
    }
}
