use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use graphene_codegen::{
    ObjectFamily, parse_object_families, render_object_id_module, render_types_mod,
};

struct ChainConfig {
    name: &'static str,
    core_path: &'static str,
    crate_path: &'static str,
}

struct GeneratedFile {
    path: PathBuf,
    contents: String,
}

const CHAINS: &[ChainConfig] = &[
    ChainConfig {
        name: "acta",
        core_path: "chains/acta-network/acta-network-core",
        crate_path: "graphene-rs/crates/graphene-chain-acta",
    },
    ChainConfig {
        name: "bitshares",
        core_path: "chains/bitshares/bitshares-core",
        crate_path: "graphene-rs/crates/graphene-chain-bitshares",
    },
    ChainConfig {
        name: "rsquared",
        core_path: "chains/rsquared/R-Squared-core",
        crate_path: "graphene-rs/crates/graphene-chain-rsquared",
    },
    ChainConfig {
        name: "swaplock",
        core_path: "chains/swaplock/swaplock-core",
        crate_path: "graphene-rs/crates/graphene-chain-swaplock",
    },
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let write = env::args().any(|argument| argument == "--write");
    let graphene_v2_root = find_graphene_v2_root()?;

    let mut total_files = 0usize;
    println!(
        "graphene-codegen object-id generation ({})",
        if write { "write" } else { "dry-run" }
    );
    println!("root: {}", graphene_v2_root.display());

    for chain in CHAINS {
        let families = read_chain_families(&graphene_v2_root, chain)?;
        let generated_files = render_chain_files(&graphene_v2_root, chain, &families);
        total_files += generated_files.len();

        println!(
            "{}: {} object families -> {} files",
            chain.name,
            families.len(),
            generated_files.len()
        );

        for generated_file in generated_files {
            if write {
                if let Some(parent) = generated_file.path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&generated_file.path, generated_file.contents)?;
            }
            println!("  {}", generated_file.path.display());
        }
    }

    if !write {
        println!("dry-run only; pass --write to update generated files");
    }
    println!("planned files: {total_files}");

    Ok(())
}

fn read_chain_families(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
) -> Result<Vec<ObjectFamily>, Box<dyn std::error::Error>> {
    let core_path = graphene_v2_root.join(chain.core_path);
    let protocol_types = core_path.join("libraries/protocol/include/graphene/protocol/types.hpp");
    let chain_types = core_path.join("libraries/chain/include/graphene/chain/types.hpp");

    let mut source = String::new();
    source.push_str(&fs::read_to_string(&protocol_types)?);
    source.push('\n');
    source.push_str(&fs::read_to_string(&chain_types)?);

    Ok(parse_object_families(&source))
}

fn render_chain_files(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
    families: &[ObjectFamily],
) -> Vec<GeneratedFile> {
    let types_path = graphene_v2_root.join(chain.crate_path).join("src/types");
    let mut files = Vec::with_capacity(families.len() + 1);

    files.push(GeneratedFile {
        path: types_path.join("mod.rs"),
        contents: render_types_mod(families),
    });

    for family in families {
        files.push(GeneratedFile {
            path: types_path.join(format!("{}.rs", family.name)),
            contents: render_object_id_module(family),
        });
    }

    files
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
