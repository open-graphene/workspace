use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use graphene_codegen::{
    WalletApiGeneration, load_codegen_config, parse_wallet_api_methods,
    render_wallet_api_spec_module,
};

struct ChainConfig {
    name: String,
    core_path: String,
    crate_path: PathBuf,
    config_path: PathBuf,
    wallet_api: Option<WalletApiGeneration>,
}

struct GeneratedFile {
    path: PathBuf,
    contents: String,
    method_count: usize,
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
        "graphene-codegen wallet API generation ({})",
        if args.write { "write" } else { "dry-run" }
    );
    println!("root: {}", graphene_v2_root.display());

    let mut total_methods = 0usize;
    let mut generated_chains = 0usize;
    for chain in &chains {
        let Some(wallet_api) = &chain.wallet_api else {
            println!("{}: wallet API skipped", chain.name);
            continue;
        };
        generated_chains += 1;
        let generated = render_chain_wallet_api(&graphene_v2_root, chain, wallet_api)?;
        total_methods += generated.method_count;

        println!("{}:", chain.name);
        println!("  config: {}", chain.config_path.display());
        println!(
            "  wallet header: {}",
            graphene_v2_root
                .join(&chain.core_path)
                .join(&wallet_api.header_path)
                .display()
        );
        println!("  methods: {}", generated.method_count);
        println!("  output: {}", generated.path.display());

        if args.write {
            if let Some(parent) = generated.path.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to create output directory {}: {error}",
                        parent.display()
                    )
                })?;
            }
            fs::write(&generated.path, generated.contents).map_err(|error| {
                format!(
                    "failed to write generated wallet API file {}: {error}",
                    generated.path.display()
                )
            })?;
        }
    }

    if !args.write {
        println!("dry-run only; pass --write to update generated files");
    }
    println!("planned wallet API chains: {generated_chains}");
    println!("planned wallet API methods: {total_methods}");

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
        fs::read_dir(&crates_path)
            .map_err(|error| {
                format!(
                    "failed to read chain crates directory {}: {error}",
                    crates_path.display()
                )
            })?
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
        if !config_path.exists() {
            return Err(format!("codegen config does not exist: {}", config_path.display()).into());
        }

        let config = load_codegen_config(&config_path).map_err(|error| {
            format!(
                "failed to load codegen config {}: {error}",
                config_path.display()
            )
        })?;
        let crate_path = config_path
            .parent()
            .ok_or_else(|| format!("{} has no parent directory", config_path.display()))?
            .to_path_buf();
        chains.push(ChainConfig {
            name: config.chain.name,
            core_path: config.chain.core_path,
            crate_path,
            config_path,
            wallet_api: config.wallet_api,
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

fn render_chain_wallet_api(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
    wallet_api: &WalletApiGeneration,
) -> Result<GeneratedFile, Box<dyn std::error::Error>> {
    let header_path = graphene_v2_root
        .join(&chain.core_path)
        .join(&wallet_api.header_path);
    let source = fs::read_to_string(&header_path).map_err(|error| {
        format!(
            "failed to read wallet API header {} for config {}: {error}",
            header_path.display(),
            chain.config_path.display()
        )
    })?;
    let report_path = header_path
        .strip_prefix(graphene_v2_root)
        .unwrap_or(&header_path)
        .display()
        .to_string();
    let methods = parse_wallet_api_methods(&source, &report_path).map_err(|error| {
        format!(
            "failed to parse wallet API from {} for config {}: {error}",
            header_path.display(),
            chain.config_path.display()
        )
    })?;
    let contents = render_wallet_api_spec_module(&chain.name, &methods);

    Ok(GeneratedFile {
        path: chain.crate_path.join(&wallet_api.output_path),
        contents,
        method_count: methods.len(),
    })
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
