use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use graphene_codegen::{
    ObjectFamily, ObjectGeneration, RustField, load_codegen_config, map_fields_to_rust,
    parse_object_families, parse_reflected_class_fields, parse_reflected_objects,
    render_object_id_module, render_object_struct, render_types_mod,
};

struct ChainConfig {
    name: String,
    core_path: String,
    crate_path: PathBuf,
    objects: Vec<ObjectGeneration>,
    marker_objects: Vec<String>,
}

struct GeneratedFile {
    path: PathBuf,
    contents: String,
}

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

    let mut total_files = 0usize;
    println!(
        "graphene-codegen object-id generation ({})",
        if args.write { "write" } else { "dry-run" }
    );
    println!("root: {}", graphene_v2_root.display());

    for chain in &chains {
        let families = read_chain_families(&graphene_v2_root, chain)?;
        let generated_objects = read_generated_object_fields(chain, &graphene_v2_root)?;
        let generated_files = render_chain_files(chain, &families, &generated_objects);
        total_files += generated_files.len();

        println!(
            "{}: {} object families -> {} files",
            chain.name,
            families.len(),
            generated_files.len()
        );

        for generated_file in generated_files {
            if args.write {
                if let Some(parent) = generated_file.path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&generated_file.path, generated_file.contents)?;
            }
            println!("  {}", generated_file.path.display());
        }
    }

    if !args.write {
        println!("dry-run only; pass --write to update generated files");
    }
    println!("planned files: {total_files}");

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
            objects: config.objects,
            marker_objects: config.marker_objects,
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

fn read_chain_families(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
) -> Result<Vec<ObjectFamily>, Box<dyn std::error::Error>> {
    let core_path = graphene_v2_root.join(&chain.core_path);
    let protocol_types = core_path.join("libraries/protocol/include/graphene/protocol/types.hpp");
    let chain_types = core_path.join("libraries/chain/include/graphene/chain/types.hpp");

    let mut source = String::new();
    source.push_str(&fs::read_to_string(&protocol_types)?);
    source.push('\n');
    source.push_str(&fs::read_to_string(&chain_types)?);

    Ok(parse_object_families(&source)?)
}

fn read_generated_object_fields(
    chain: &ChainConfig,
    graphene_v2_root: &Path,
) -> Result<BTreeMap<String, Vec<RustField>>, Box<dyn std::error::Error>> {
    let mut generated_objects = BTreeMap::new();

    for marker_object in &chain.marker_objects {
        generated_objects.insert(marker_object.clone(), Vec::new());
    }

    for object in &chain.objects {
        if let Some(fields) = read_object_fields(graphene_v2_root, chain, object)? {
            generated_objects.insert(object.family_name.clone(), fields);
        }
    }

    Ok(generated_objects)
}

fn read_object_fields(
    graphene_v2_root: &Path,
    chain: &ChainConfig,
    object: &ObjectGeneration,
) -> Result<Option<Vec<RustField>>, Box<dyn std::error::Error>> {
    let core_path = graphene_v2_root.join(&chain.core_path);
    let header_path = core_path.join(&object.header_path);
    let source_path = core_path.join(&object.source_path);

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
        parse_reflected_class_fields(&header_source, &object.cpp_class, &reflected_fields)?;

    Ok(Some(map_fields_to_rust(&cpp_fields)?))
}

fn render_chain_files(
    chain: &ChainConfig,
    families: &[ObjectFamily],
    generated_objects: &BTreeMap<String, Vec<RustField>>,
) -> Vec<GeneratedFile> {
    let types_path = chain.crate_path.join("src/types");
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
