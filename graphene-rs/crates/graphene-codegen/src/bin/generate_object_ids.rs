use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use graphene_codegen::{
    ObjectFamily, RustField, map_fields_to_rust, parse_object_families,
    parse_reflected_class_fields, parse_reflected_objects, render_object_id_module,
    render_object_struct, render_types_mod,
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

struct ObjectGeneration {
    family_name: &'static str,
    cpp_class: &'static str,
    header_path: &'static str,
    source_path: &'static str,
}

const GENERATED_OBJECTS: &[ObjectGeneration] = &[
    ObjectGeneration {
        family_name: "account_balance",
        cpp_class: "account_balance_object",
        header_path: "libraries/chain/include/graphene/chain/account_object.hpp",
        source_path: "libraries/chain/account_object.cpp",
    },
    ObjectGeneration {
        family_name: "account_history",
        cpp_class: "account_history_object",
        header_path: "libraries/chain/include/graphene/chain/operation_history_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "asset_dynamic_data",
        cpp_class: "asset_dynamic_data_object",
        header_path: "libraries/chain/include/graphene/chain/asset_object.hpp",
        source_path: "libraries/chain/asset_object.cpp",
    },
    ObjectGeneration {
        family_name: "blinded_balance",
        cpp_class: "blinded_balance_object",
        header_path: "libraries/chain/include/graphene/chain/confidential_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "block_summary",
        cpp_class: "block_summary_object",
        header_path: "libraries/chain/include/graphene/chain/block_summary_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "buyback",
        cpp_class: "buyback_object",
        header_path: "libraries/chain/include/graphene/chain/buyback_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "committee_member",
        cpp_class: "committee_member_object",
        header_path: "libraries/chain/include/graphene/chain/committee_member_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "dynamic_global_property",
        cpp_class: "dynamic_global_property_object",
        header_path: "libraries/chain/include/graphene/chain/global_property_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "fba_accumulator",
        cpp_class: "fba_accumulator_object",
        header_path: "libraries/chain/include/graphene/chain/fba_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "witness",
        cpp_class: "witness_object",
        header_path: "libraries/chain/include/graphene/chain/witness_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "witness_schedule",
        cpp_class: "witness_schedule_object",
        header_path: "libraries/chain/include/graphene/chain/witness_schedule_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "withdraw_permission",
        cpp_class: "withdraw_permission_object",
        header_path: "libraries/chain/include/graphene/chain/withdraw_permission_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
];

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
        let generated_objects = read_generated_object_fields(&graphene_v2_root, chain)?;
        let generated_files =
            render_chain_files(&graphene_v2_root, chain, &families, &generated_objects);
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

    Ok(parse_object_families(&source)?)
}

fn read_generated_object_fields(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
) -> Result<BTreeMap<String, Vec<RustField>>, Box<dyn std::error::Error>> {
    let mut generated_objects = BTreeMap::new();

    for object in GENERATED_OBJECTS {
        if let Some(fields) = read_object_fields(graphene_v2_root, chain, object)? {
            generated_objects.insert(object.family_name.to_owned(), fields);
        }
    }

    Ok(generated_objects)
}

fn read_object_fields(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
    object: &ObjectGeneration,
) -> Result<Option<Vec<RustField>>, Box<dyn std::error::Error>> {
    let core_path = graphene_v2_root.join(chain.core_path);
    let header_path = core_path.join(object.header_path);
    let source_path = core_path.join(object.source_path);

    if !header_path.exists() || !source_path.exists() {
        return Ok(None);
    }

    let header_source = fs::read_to_string(&header_path)?;
    let source = fs::read_to_string(&source_path)?;
    let reflected_objects = parse_reflected_objects(&source)?;
    let reflected_cpp_type = format!("graphene::chain::{}", object.cpp_class);
    let Some(reflected_object) = reflected_objects
        .iter()
        .find(|reflected| reflected.cpp_type == reflected_cpp_type)
    else {
        return Ok(None);
    };

    let reflected_fields = reflected_object
        .fields
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let cpp_fields =
        parse_reflected_class_fields(&header_source, object.cpp_class, &reflected_fields)?;

    Ok(Some(map_fields_to_rust(&cpp_fields)?))
}

fn render_chain_files(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
    families: &[ObjectFamily],
    generated_objects: &BTreeMap<String, Vec<RustField>>,
) -> Vec<GeneratedFile> {
    let types_path = graphene_v2_root.join(chain.crate_path).join("src/types");
    let mut files = Vec::with_capacity(families.len() + 1);

    files.push(GeneratedFile {
        path: types_path.join("mod.rs"),
        contents: render_types_mod(families),
    });

    for family in families {
        let mut contents = render_object_id_module(family);
        if let Some(fields) = generated_objects.get(&family.name) {
            contents.push('\n');
            contents.push_str(&render_object_struct(fields));
        }

        files.push(GeneratedFile {
            path: types_path.join(format!("{}.rs", family.name)),
            contents,
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
