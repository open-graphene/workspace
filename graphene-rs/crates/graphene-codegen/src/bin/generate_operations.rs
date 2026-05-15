use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use graphene_codegen::{
    OperationDeclarationError, ProtocolHeaderSource, load_codegen_config,
    parse_operation_declarations, parse_operation_variants, render_operation_model_skips_report,
    render_operation_structs_module,
};

struct ChainConfig {
    name: String,
    core_path: String,
    crate_path: PathBuf,
    config_path: PathBuf,
}

struct GeneratedOperations {
    operations_path: PathBuf,
    operations_source: String,
    report_path: PathBuf,
    report_source: String,
    variant_count: usize,
    generated_struct_count: usize,
    skipped_unsupported_count: usize,
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
        "graphene-codegen operation struct generation ({})",
        if args.write { "write" } else { "dry-run" }
    );
    println!("root: {}", graphene_v2_root.display());

    let mut total_variants = 0usize;
    let mut total_generated_structs = 0usize;
    let mut total_skipped_unsupported = 0usize;

    for chain in &chains {
        let generated = render_chain_operations(&graphene_v2_root, chain)?;
        total_variants += generated.variant_count;
        total_generated_structs += generated.generated_struct_count;
        total_skipped_unsupported += generated.skipped_unsupported_count;

        println!("{}:", chain.name);
        println!("  mode: {}", if args.write { "write" } else { "dry-run" });
        println!("  config: {}", chain.config_path.display());
        println!(
            "  protocol include dir: {}",
            protocol_include_dir(&graphene_v2_root, chain).display()
        );
        println!("  operation variants: {}", generated.variant_count);
        println!("  generated structs: {}", generated.generated_struct_count);
        println!(
            "  skipped/unsupported rows: {}",
            generated.skipped_unsupported_count
        );
        println!(
            "  operations output: {}",
            generated.operations_path.display()
        );
        println!("  report: {}", generated.report_path.display());

        write_generated_file(
            &generated.operations_path,
            &generated.operations_source,
            args.write,
        )?;
        write_generated_file(&generated.report_path, &generated.report_source, args.write)?;
    }

    if !args.write {
        println!("dry-run only; pass --write to update generated files");
    }
    println!("planned operation variants: {total_variants}");
    println!("planned generated structs: {total_generated_structs}");
    println!("planned skipped/unsupported rows: {total_skipped_unsupported}");

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

fn render_chain_operations(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
) -> Result<GeneratedOperations, Box<dyn std::error::Error>> {
    let operations_header_path = operations_header_path(graphene_v2_root, chain);
    let operations_source = fs::read_to_string(&operations_header_path).map_err(|error| {
        format!(
            "failed to read operations header {} for config {}: {error}",
            operations_header_path.display(),
            chain.config_path.display()
        )
    })?;
    let variants = parse_operation_variants(&operations_source).map_err(|error| {
        format!(
            "failed to parse operation variants from {} for config {}: {error}",
            operations_header_path.display(),
            chain.config_path.display()
        )
    })?;

    let header_sources = read_protocol_headers(graphene_v2_root, chain)?;
    let header_refs = header_sources
        .iter()
        .map(|header| ProtocolHeaderSource {
            path: header.path.as_str(),
            source: header.source.as_str(),
        })
        .collect::<Vec<_>>();
    let declarations = parse_operation_declarations(&variants, &header_refs)
        .map_err(|errors| format_declaration_errors(&chain.config_path, &errors))?;
    let rendered = render_operation_structs_module(&declarations);
    let report_source = render_operation_model_skips_report(
        variants.len(),
        rendered.generated_struct_count,
        &rendered.report_rows,
    );

    Ok(GeneratedOperations {
        operations_path: chain.crate_path.join("src/operations.rs"),
        operations_source: rendered.source,
        report_path: chain.crate_path.join("src/operation_model_skips.md"),
        report_source,
        variant_count: variants.len(),
        generated_struct_count: rendered.generated_struct_count,
        skipped_unsupported_count: rendered.unsupported_count,
    })
}

struct OwnedProtocolHeaderSource {
    path: String,
    source: String,
}

fn read_protocol_headers(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
) -> Result<Vec<OwnedProtocolHeaderSource>, Box<dyn std::error::Error>> {
    let include_dir = protocol_include_dir(graphene_v2_root, chain);
    let mut header_paths = fs::read_dir(&include_dir)
        .map_err(|error| {
            format!(
                "failed to read protocol include dir {} for config {}: {error}",
                include_dir.display(),
                chain.config_path.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("hpp"))
        .collect::<Vec<_>>();
    header_paths.sort();

    let mut headers = Vec::with_capacity(header_paths.len());
    for path in header_paths {
        let source = fs::read_to_string(&path).map_err(|error| {
            format!(
                "failed to read protocol header {} for config {}: {error}",
                path.display(),
                chain.config_path.display()
            )
        })?;
        let report_path = path
            .strip_prefix(graphene_v2_root)
            .unwrap_or(&path)
            .display()
            .to_string();
        headers.push(OwnedProtocolHeaderSource {
            path: report_path,
            source,
        });
    }

    Ok(headers)
}

fn protocol_include_dir(graphene_v2_root: &Path, chain: &ChainConfig) -> PathBuf {
    graphene_v2_root
        .join(&chain.core_path)
        .join("libraries/protocol/include/graphene/protocol")
}

fn operations_header_path(graphene_v2_root: &Path, chain: &ChainConfig) -> PathBuf {
    protocol_include_dir(graphene_v2_root, chain).join("operations.hpp")
}

fn format_declaration_errors(config_path: &Path, errors: &[OperationDeclarationError]) -> String {
    let mut message = format!(
        "failed to parse operation declarations for config {} ({} errors)",
        config_path.display(),
        errors.len()
    );

    for error in errors.iter().take(10) {
        message.push_str(&format!(
            "\n- operation={} tag={} field={} cpp_type={} source={} reason={}",
            error.name,
            error.tag,
            error.field_name.as_deref().unwrap_or("<operation>"),
            error.cpp_type_hint.as_deref().unwrap_or(&error.cpp_type),
            source_location(error),
            error.reason
        ));
    }

    if errors.len() > 10 {
        message.push_str(&format!("\n- ... {} more errors", errors.len() - 10));
    }

    message
}

fn source_location(error: &OperationDeclarationError) -> String {
    match (&error.source_file, error.source_line) {
        (Some(file), Some(line)) => format!("{file}:{line}"),
        (Some(file), None) => file.clone(),
        (None, Some(line)) => format!("<unknown>:{line}"),
        (None, None) => "<unknown>".to_owned(),
    }
}

fn write_generated_file(
    path: &Path,
    contents: &str,
    write: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !write {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create output directory {}: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(path, contents)
        .map_err(|error| format!("failed to write generated file {}: {error}", path.display()))?;

    Ok(())
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
    use super::{Args, resolve_config_path, write_generated_file};
    use std::fs;
    use std::path::{Path, PathBuf};

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

    #[test]
    fn resolves_relative_config_paths_from_graphene_rs_root() {
        assert_eq!(
            resolve_config_path(
                Path::new("/repo/graphene-v2"),
                Path::new("crates/graphene-chain-bitshares/codegen.toml")
            ),
            PathBuf::from(
                "/repo/graphene-v2/graphene-rs/crates/graphene-chain-bitshares/codegen.toml"
            )
        );
    }

    #[test]
    fn dry_run_does_not_write_output_file() {
        let path = unique_temp_path().join("nested/operations.rs");
        write_generated_file(&path, "generated", false).expect("dry-run should succeed");

        assert!(!path.exists());
    }

    #[test]
    fn write_mode_creates_parent_directories() {
        let dir = unique_temp_path();
        let path = dir.join("nested/operations.rs");
        write_generated_file(&path, "generated", true).expect("write should succeed");

        assert_eq!(
            fs::read_to_string(&path).expect("generated file"),
            "generated"
        );
        fs::remove_dir_all(dir).expect("cleanup temp output");
    }

    fn unique_temp_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "graphene-codegen-generate-operations-test-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("unnamed")
        ))
    }
}
