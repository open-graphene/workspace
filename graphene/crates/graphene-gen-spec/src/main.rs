use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value as JsonValue};
use toml::Value as TomlValue;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    match args.next().as_deref().and_then(|arg| arg.to_str()) {
        Some("generate") => {
            let config = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("graphene/chains/swaplock.toml"));
            if let Some(extra) = args.next() {
                return Err(format!("unexpected argument: {}", extra.to_string_lossy()).into());
            }
            generate(&config)
        }
        Some("audit") => {
            let config = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("graphene/chains/swaplock.toml"));
            if let Some(extra) = args.next() {
                return Err(format!("unexpected argument: {}", extra.to_string_lossy()).into());
            }
            audit(&config)
        }
        _ => Err("usage: graphene-gen-spec <generate|audit> [config.toml]".into()),
    }
}

#[derive(Debug)]
struct LoadedConfig {
    surfaces: Vec<SpecConfig>,
    networks: Vec<NetworkConfig>,
}

#[derive(Debug)]
struct BundleConfig {
    chain_name: String,
    manifest_output: PathBuf,
    surfaces: Vec<SpecConfig>,
    networks: Vec<NetworkConfig>,
}

#[derive(Debug)]
struct NetworkConfig {
    name: String,
    label: Option<String>,
    is_default: bool,
    chain_id: Option<String>,
    address_prefix: Option<String>,
    core_asset: Option<CoreAssetConfig>,
    endpoints: Vec<EndpointConfig>,
}

#[derive(Debug)]
struct CoreAssetConfig {
    asset_id: Option<String>,
    symbol: Option<String>,
    precision: Option<u64>,
}

#[derive(Debug)]
struct EndpointConfig {
    url: String,
    transport: String,
    label: Option<String>,
    priority: Option<i64>,
}

#[derive(Debug)]
struct SpecConfig {
    chain_name: String,
    core_root: PathBuf,
    surface_name: String,
    surface_kind: String,
    api_header: String,
    api_qualified_name: String,
    excluded_methods: Vec<String>,
    header_roots: Vec<String>,
    output: PathBuf,
}

fn generate(config_path: &Path) -> Result<(), Box<dyn Error>> {
    let workspace_root = find_workspace_root(env::current_dir()?)?;
    let bundle = build_bundle(load_config(&workspace_root, config_path)?)?;
    let openrpc_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("openrpc");

    for config in &bundle.surfaces {
        run(Command::new(openrpc_dir.join("gen_openrpc.sh"))
            .arg("--chain-name")
            .arg(&config.chain_name)
            .arg("--core-root")
            .arg(&config.core_root)
            .arg("--api-header")
            .arg(&config.api_header)
            .arg("--api-qualified-name")
            .arg(&config.api_qualified_name)
            .args(
                config.excluded_methods.iter().flat_map(|method| {
                    [OsString::from("--exclude-method"), OsString::from(method)]
                }),
            )
            .arg("--output")
            .arg(&config.output)
            .args(
                config
                    .header_roots
                    .iter()
                    .flat_map(|root| [OsString::from("--header-root"), OsString::from(root)]),
            ))?;

        enrich_open_graphene_spec(config)?;
        println!("open-graphene surface spec: {}", config.output.display());
    }

    write_manifest(&bundle)?;
    println!(
        "open-graphene manifest: {}",
        bundle.manifest_output.display()
    );

    Ok(())
}

fn audit(config_path: &Path) -> Result<(), Box<dyn Error>> {
    let workspace_root = find_workspace_root(env::current_dir()?)?;
    let bundle = build_bundle(load_config(&workspace_root, config_path)?)?;
    let mut failed = false;

    if let Err(error) = audit_manifest(&bundle) {
        eprintln!("audit failed for {} manifest: {error}", bundle.chain_name);
        failed = true;
    }

    for config in &bundle.surfaces {
        if let Err(error) = audit_surface_spec(config) {
            eprintln!("audit failed for {}: {error}", config.surface_name);
            failed = true;
        }
    }

    if failed {
        Err("one or more open-graphene specs failed audit".into())
    } else {
        Ok(())
    }
}

fn build_bundle(loaded: LoadedConfig) -> Result<BundleConfig, Box<dyn Error>> {
    let configs = loaded.surfaces;
    let first = configs.first().ok_or("no surface configs loaded")?;
    let chain_name = first.chain_name.clone();
    let manifest_dir = first
        .output
        .parent()
        .ok_or_else(|| format!("surface output has no parent: {}", first.output.display()))?
        .to_path_buf();

    for config in &configs {
        if config.chain_name != chain_name {
            return Err(format!(
                "all surfaces in one bundle must use chain {chain_name}, got {}",
                config.chain_name
            )
            .into());
        }
        let output_dir = config
            .output
            .parent()
            .ok_or_else(|| format!("surface output has no parent: {}", config.output.display()))?;
        if output_dir != manifest_dir {
            return Err(format!(
                "all surface specs for chain {chain_name} must share one directory; {} is outside {}",
                config.output.display(),
                manifest_dir.display()
            )
            .into());
        }
    }

    Ok(BundleConfig {
        chain_name,
        manifest_output: manifest_dir.join("open-graphene.json"),
        surfaces: configs,
        networks: loaded.networks,
    })
}

fn audit_surface_spec(config: &SpecConfig) -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(&config.output).map_err(|error| {
        format!(
            "failed to read open-graphene surface spec {}: {error}",
            config.output.display()
        )
    })?;
    let spec: JsonValue = serde_json::from_str(&source).map_err(|error| {
        format!(
            "failed to parse open-graphene surface spec {}: {error}",
            config.output.display()
        )
    })?;

    let mut failures = Vec::new();
    let mut warnings = Vec::new();

    if spec.get("openrpc").and_then(JsonValue::as_str) != Some("1.3.2") {
        failures.push("openrpc must be 1.3.2".to_owned());
    }

    let profile = spec.get("x-open-graphene").and_then(JsonValue::as_object);
    if profile.is_none() {
        failures.push("missing top-level x-open-graphene profile metadata".to_owned());
    }

    let methods = spec
        .get("methods")
        .and_then(JsonValue::as_array)
        .ok_or("spec is missing methods array")?;
    let schemas = spec
        .pointer("/components/schemas")
        .and_then(JsonValue::as_object)
        .ok_or("spec is missing components.schemas object")?;

    if methods.is_empty() {
        warnings.push("methods array is empty".to_owned());
    }
    if schemas.is_empty() {
        warnings.push("components.schemas is empty".to_owned());
    }

    let unresolved = count_todo_descriptions(&spec);
    if unresolved > 0 {
        failures.push(format!(
            "spec contains {unresolved} TODO placeholder descriptions"
        ));
    }

    let refs = collect_refs(&spec);
    let missing_refs = refs
        .iter()
        .filter_map(|reference| {
            reference
                .strip_prefix("#/components/schemas/")
                .filter(|name| !schemas.contains_key(*name))
                .map(|name| format!("{reference} ({name})"))
        })
        .collect::<Vec<_>>();
    if !missing_refs.is_empty() {
        failures.push(format!(
            "unresolved component refs: {}",
            missing_refs
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let static_variants = schemas
        .values()
        .filter(|schema| is_static_variant_schema(schema))
        .count();
    let rust_extensions = count_key(&spec, "x-rust-type");
    if rust_extensions > 0 {
        warnings.push(format!(
            "spec contains {rust_extensions} legacy x-rust-type extension(s); keep for compatibility until scalar metadata is migrated"
        ));
    }

    println!("audit surface {}", config.surface_name);
    println!("  output: {}", config.output.display());
    println!("  methods: {}", methods.len());
    println!("  schemas: {}", schemas.len());
    println!("  static variants: {static_variants}");
    println!("  refs: {}", refs.len());
    println!("  TODO placeholders: {unresolved}");
    for warning in &warnings {
        println!("  warning: {warning}");
    }

    if failures.is_empty() {
        println!("  status: pass");
        Ok(())
    } else {
        println!("  status: fail");
        for failure in &failures {
            println!("  failure: {failure}");
        }
        Err(format!("{} failed audit", config.surface_name).into())
    }
}

fn audit_manifest(bundle: &BundleConfig) -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(&bundle.manifest_output).map_err(|error| {
        format!(
            "failed to read open-graphene manifest {}: {error}",
            bundle.manifest_output.display()
        )
    })?;
    let manifest: JsonValue = serde_json::from_str(&source).map_err(|error| {
        format!(
            "failed to parse open-graphene manifest {}: {error}",
            bundle.manifest_output.display()
        )
    })?;

    let mut failures = Vec::new();
    let mut warnings = Vec::new();
    let profile = manifest
        .get("x-open-graphene")
        .and_then(JsonValue::as_object);

    let Some(profile) = profile else {
        println!("audit manifest {}", bundle.chain_name);
        println!("  output: {}", bundle.manifest_output.display());
        println!("  status: fail");
        println!("  failure: missing top-level x-open-graphene profile metadata");
        return Err(format!("{} manifest failed audit", bundle.chain_name).into());
    };

    if profile.get("kind").and_then(JsonValue::as_str) != Some("chain-bundle") {
        failures.push("x-open-graphene.kind must be chain-bundle".to_owned());
    }
    if profile.get("version").and_then(JsonValue::as_str) != Some("0.1.0") {
        failures.push("x-open-graphene.version must be 0.1.0".to_owned());
    }
    if profile.get("chain").and_then(JsonValue::as_str) != Some(bundle.chain_name.as_str()) {
        failures.push(format!(
            "x-open-graphene.chain must be {}",
            bundle.chain_name
        ));
    }

    let surfaces = profile
        .get("surfaces")
        .and_then(JsonValue::as_array)
        .ok_or("manifest is missing x-open-graphene.surfaces array")?;
    if surfaces.len() != bundle.surfaces.len() {
        failures.push(format!(
            "manifest lists {} surfaces, config has {}",
            surfaces.len(),
            bundle.surfaces.len()
        ));
    }

    let manifest_dir = bundle.manifest_output.parent().ok_or_else(|| {
        format!(
            "manifest has no parent: {}",
            bundle.manifest_output.display()
        )
    })?;

    audit_networks(profile, bundle, &mut failures, &mut warnings)?;

    for config in &bundle.surfaces {
        let entry = surfaces.iter().find(|surface| {
            surface
                .get("name")
                .and_then(JsonValue::as_str)
                .is_some_and(|name| name == config.surface_name)
        });
        let Some(entry) = entry else {
            failures.push(format!(
                "manifest is missing surface {}",
                config.surface_name
            ));
            continue;
        };

        if entry.get("kind").and_then(JsonValue::as_str) != Some(config.surface_kind.as_str()) {
            failures.push(format!(
                "surface {} kind mismatch in manifest",
                config.surface_name
            ));
        }
        if entry.get("apiQualifiedName").and_then(JsonValue::as_str)
            != Some(config.api_qualified_name.as_str())
        {
            failures.push(format!(
                "surface {} apiQualifiedName mismatch in manifest",
                config.surface_name
            ));
        }

        let Some(spec_path) = entry.get("spec").and_then(JsonValue::as_str) else {
            failures.push(format!(
                "surface {} is missing spec path",
                config.surface_name
            ));
            continue;
        };
        if let Err(error) = validate_relative_spec_path(spec_path) {
            failures.push(format!(
                "surface {} has invalid spec path {spec_path}: {error}",
                config.surface_name
            ));
            continue;
        }
        let resolved = normalize(&manifest_dir.join(spec_path));
        if resolved != config.output {
            failures.push(format!(
                "surface {} spec path resolves to {}, expected {}",
                config.surface_name,
                resolved.display(),
                config.output.display()
            ));
        }
        if !resolved.is_file() {
            failures.push(format!(
                "surface {} spec file does not exist: {}",
                config.surface_name,
                resolved.display()
            ));
        }
    }

    println!("audit manifest {}", bundle.chain_name);
    println!("  output: {}", bundle.manifest_output.display());
    println!("  surfaces: {}", surfaces.len());
    println!("  networks: {}", bundle.networks.len());
    for warning in &warnings {
        println!("  warning: {warning}");
    }

    if failures.is_empty() {
        println!("  status: pass");
        Ok(())
    } else {
        println!("  status: fail");
        for failure in &failures {
            println!("  failure: {failure}");
        }
        Err(format!("{} manifest failed audit", bundle.chain_name).into())
    }
}

fn audit_networks(
    profile: &serde_json::Map<String, JsonValue>,
    bundle: &BundleConfig,
    failures: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let networks = profile
        .get("networks")
        .and_then(JsonValue::as_array)
        .ok_or("manifest is missing x-open-graphene.networks array")?;

    if networks.len() != bundle.networks.len() {
        failures.push(format!(
            "manifest lists {} networks, config has {}",
            networks.len(),
            bundle.networks.len()
        ));
    }
    if networks.is_empty() {
        warnings.push("manifest has no named networks".to_owned());
    }

    let mut names = std::collections::BTreeSet::new();
    let mut default_count = 0usize;
    for network in networks {
        let Some(name) = network.get("name").and_then(JsonValue::as_str) else {
            failures.push("network entry is missing name".to_owned());
            continue;
        };
        if !is_stable_network_name(name) {
            failures.push(format!(
                "network name {name} must use lowercase letters, digits, hyphen, or underscore"
            ));
        }
        if !names.insert(name.to_owned()) {
            failures.push(format!("duplicate network name {name}"));
        }
        if network.get("default").and_then(JsonValue::as_bool) == Some(true) {
            default_count += 1;
        }
        if network.get("chainId").and_then(JsonValue::as_str).is_none() {
            warnings.push(format!("network {name} is missing chainId"));
        }
        if network
            .get("addressPrefix")
            .and_then(JsonValue::as_str)
            .is_none()
        {
            warnings.push(format!("network {name} is missing addressPrefix"));
        }
        if network
            .get("coreAsset")
            .and_then(JsonValue::as_object)
            .is_none()
        {
            warnings.push(format!("network {name} is missing coreAsset"));
        }
        if let Some(endpoints) = network.get("endpoints").and_then(JsonValue::as_array) {
            for endpoint in endpoints {
                if endpoint.get("url").and_then(JsonValue::as_str).is_none() {
                    failures.push(format!("network {name} has endpoint without url"));
                }
                if endpoint
                    .get("transport")
                    .and_then(JsonValue::as_str)
                    .is_none()
                {
                    failures.push(format!("network {name} has endpoint without transport"));
                }
            }
        }
    }
    if default_count > 1 {
        failures.push(format!(
            "manifest has {default_count} default networks; expected at most one"
        ));
    }

    Ok(())
}

fn is_stable_network_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
}

fn validate_relative_spec_path(spec_path: &str) -> Result<(), String> {
    let path = Path::new(spec_path);
    if spec_path.is_empty() {
        return Err("path is empty".to_owned());
    }
    if path.is_absolute() {
        return Err("path must be relative".to_owned());
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("path must not contain ..".to_owned());
    }
    Ok(())
}

fn network_json(network: &NetworkConfig) -> JsonValue {
    let mut object = serde_json::Map::new();
    object.insert("name".to_owned(), json!(network.name));
    if let Some(label) = &network.label {
        object.insert("label".to_owned(), json!(label));
    }
    if network.is_default {
        object.insert("default".to_owned(), json!(true));
    }
    if let Some(chain_id) = &network.chain_id {
        object.insert("chainId".to_owned(), json!(chain_id));
    }
    if let Some(address_prefix) = &network.address_prefix {
        object.insert("addressPrefix".to_owned(), json!(address_prefix));
    }
    if let Some(core_asset) = &network.core_asset {
        let mut asset = serde_json::Map::new();
        if let Some(asset_id) = &core_asset.asset_id {
            asset.insert("assetId".to_owned(), json!(asset_id));
        }
        if let Some(symbol) = &core_asset.symbol {
            asset.insert("symbol".to_owned(), json!(symbol));
        }
        if let Some(precision) = core_asset.precision {
            asset.insert("precision".to_owned(), json!(precision));
        }
        object.insert("coreAsset".to_owned(), JsonValue::Object(asset));
    }
    if !network.endpoints.is_empty() {
        object.insert(
            "endpoints".to_owned(),
            JsonValue::Array(network.endpoints.iter().map(endpoint_json).collect()),
        );
    }
    JsonValue::Object(object)
}

fn endpoint_json(endpoint: &EndpointConfig) -> JsonValue {
    let mut object = serde_json::Map::new();
    object.insert("url".to_owned(), json!(endpoint.url));
    object.insert("transport".to_owned(), json!(endpoint.transport));
    if let Some(label) = &endpoint.label {
        object.insert("label".to_owned(), json!(label));
    }
    if let Some(priority) = endpoint.priority {
        object.insert("priority".to_owned(), json!(priority));
    }
    JsonValue::Object(object)
}

fn write_manifest(bundle: &BundleConfig) -> Result<(), Box<dyn Error>> {
    let manifest_dir = bundle.manifest_output.parent().ok_or_else(|| {
        format!(
            "manifest has no parent: {}",
            bundle.manifest_output.display()
        )
    })?;

    let surfaces = bundle
        .surfaces
        .iter()
        .map(|surface| {
            let spec = relative_child_path(manifest_dir, &surface.output)?;
            Ok(json!({
                "name": surface.surface_name,
                "kind": surface.surface_kind,
                "apiQualifiedName": surface.api_qualified_name,
                "spec": spec,
            }))
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    let networks = bundle.networks.iter().map(network_json).collect::<Vec<_>>();

    let manifest = json!({
        "x-open-graphene": {
            "version": "0.1.0",
            "kind": "chain-bundle",
            "chain": bundle.chain_name,
            "networks": networks,
            "surfaces": surfaces,
        }
    });

    fs::create_dir_all(manifest_dir)?;
    fs::write(
        &bundle.manifest_output,
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;
    Ok(())
}

fn relative_child_path(base: &Path, child: &Path) -> Result<String, Box<dyn Error>> {
    let relative = child.strip_prefix(base).map_err(|_| {
        format!(
            "{} is not inside manifest directory {}",
            child.display(),
            base.display()
        )
    })?;
    let value = relative.to_string_lossy().replace('\\', "/");
    validate_relative_spec_path(&value)?;
    Ok(value)
}

fn enrich_open_graphene_spec(config: &SpecConfig) -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(&config.output)?;
    let mut spec: JsonValue = serde_json::from_str(&source)?;
    let object = spec
        .as_object_mut()
        .ok_or("generated OpenRPC document is not a JSON object")?;

    object.insert(
        "x-open-graphene".to_owned(),
        json!({
            "version": "0.1.0",
            "kind": "api-surface",
            "chain": config.chain_name,
            "surface": {
                "name": config.surface_name,
                "kind": config.surface_kind,
                "apiQualifiedName": config.api_qualified_name,
            },
            "rpc": {
                "paramsEncoding": "positional"
            },
            "source": {
                "coreRoot": config.core_root,
                "apiHeader": config.api_header,
                "headerRoots": config.header_roots,
                "excludedMethods": config.excluded_methods,
            }
        }),
    );

    fs::write(&config.output, serde_json::to_string_pretty(&spec)? + "\n")?;
    Ok(())
}

fn load_config(workspace_root: &Path, config_path: &Path) -> Result<LoadedConfig, Box<dyn Error>> {
    let config_path = absolutize(workspace_root, config_path);
    let config_dir = config_path
        .parent()
        .ok_or_else(|| format!("config has no parent directory: {}", config_path.display()))?;
    let source = fs::read_to_string(&config_path)?;
    let config: TomlValue = toml::from_str(&source)?;

    let chain = table(&config, "chain")?;
    let chain_name = string(chain, "name")?.to_owned();
    let core_root = resolve(config_dir, string(chain, "core_root")?);

    let open_graphene = config
        .get("open_graphene")
        .or_else(|| config.get("openrpc"))
        .and_then(TomlValue::as_table);
    let inherited_header_roots = open_graphene
        .map(|table| string_array(table, "header_roots", "header_roots"))
        .transpose()?
        .flatten()
        .unwrap_or_default();

    let surfaces = config
        .get("surfaces")
        .and_then(TomlValue::as_array)
        .ok_or("missing [[surfaces]] entries")?;
    if surfaces.is_empty() {
        return Err("[[surfaces]] must contain at least one surface".into());
    }

    let surfaces = surfaces
        .iter()
        .map(|surface| {
            let surface = surface
                .as_table()
                .ok_or("[[surfaces]] entries must be tables")?;
            load_surface_config(
                &chain_name,
                &core_root,
                surface,
                config_dir,
                &inherited_header_roots,
            )
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let networks = load_networks(&config)?;

    Ok(LoadedConfig { surfaces, networks })
}

fn load_networks(config: &TomlValue) -> Result<Vec<NetworkConfig>, Box<dyn Error>> {
    let Some(networks) = config.get("networks").and_then(TomlValue::as_array) else {
        return Ok(Vec::new());
    };

    networks
        .iter()
        .map(|network| {
            let network = network
                .as_table()
                .ok_or("[[networks]] entries must be tables")?;
            let endpoints = network
                .get("endpoints")
                .and_then(TomlValue::as_array)
                .map(|endpoints| {
                    endpoints
                        .iter()
                        .map(|endpoint| {
                            let endpoint = endpoint
                                .as_table()
                                .ok_or("[[networks.endpoints]] entries must be tables")?;
                            Ok(EndpointConfig {
                                url: string(endpoint, "url")?.to_owned(),
                                transport: endpoint
                                    .get("transport")
                                    .and_then(TomlValue::as_str)
                                    .unwrap_or("websocket")
                                    .to_owned(),
                                label: optional_string(endpoint, "label"),
                                priority: optional_i64(endpoint, "priority")?,
                            })
                        })
                        .collect::<Result<Vec<_>, Box<dyn Error>>>()
                })
                .transpose()?
                .unwrap_or_default();

            let core_asset = network
                .get("core_asset")
                .and_then(TomlValue::as_table)
                .map(|core_asset| -> Result<CoreAssetConfig, Box<dyn Error>> {
                    Ok(CoreAssetConfig {
                        asset_id: optional_string(core_asset, "asset_id"),
                        symbol: optional_string(core_asset, "symbol"),
                        precision: optional_u64(core_asset, "precision")?,
                    })
                })
                .transpose()?;

            Ok(NetworkConfig {
                name: string(network, "name")?.to_owned(),
                label: optional_string(network, "label"),
                is_default: optional_bool(network, "default")?.unwrap_or(false),
                chain_id: optional_string(network, "chain_id"),
                address_prefix: optional_string(network, "address_prefix"),
                core_asset,
                endpoints,
            })
        })
        .collect()
}

fn load_surface_config(
    chain_name: &str,
    core_root: &Path,
    surface: &toml::map::Map<String, TomlValue>,
    config_dir: &Path,
    inherited_header_roots: &[String],
) -> Result<SpecConfig, Box<dyn Error>> {
    let surface_name = surface
        .get("name")
        .and_then(TomlValue::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(chain_name)
        .to_owned();
    let surface_kind = surface
        .get("kind")
        .and_then(TomlValue::as_str)
        .unwrap_or("unspecified")
        .to_owned();
    let api_header = string(surface, "api_header")?.to_owned();
    let api_qualified_name = string(surface, "api_qualified_name")?.to_owned();
    let excluded_methods =
        string_array(surface, "excluded_methods", "excluded_methods")?.unwrap_or_default();
    let header_roots = string_array(surface, "header_roots", "header_roots")?
        .unwrap_or_else(|| inherited_header_roots.to_vec());
    if header_roots.is_empty() {
        return Err(format!("surface {surface_name} has no header_roots").into());
    }

    Ok(SpecConfig {
        chain_name: chain_name.to_owned(),
        core_root: core_root.to_path_buf(),
        surface_name,
        surface_kind,
        api_header,
        api_qualified_name,
        excluded_methods,
        header_roots,
        output: resolve(config_dir, string(surface, "output")?),
    })
}

fn run(command: &mut Command) -> Result<(), Box<dyn Error>> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed with {status}: {command:?}").into())
    }
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
        .ok_or_else(|| format!("missing string field {key}").into())
}

fn optional_string(table: &toml::map::Map<String, TomlValue>, key: &str) -> Option<String> {
    table
        .get(key)
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned)
}

fn optional_bool(
    table: &toml::map::Map<String, TomlValue>,
    key: &str,
) -> Result<Option<bool>, Box<dyn Error>> {
    table
        .get(key)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| format!("{key} must be a boolean"))
        })
        .transpose()
        .map_err(Into::into)
}

fn optional_i64(
    table: &toml::map::Map<String, TomlValue>,
    key: &str,
) -> Result<Option<i64>, Box<dyn Error>> {
    table
        .get(key)
        .map(|value| {
            value
                .as_integer()
                .ok_or_else(|| format!("{key} must be an integer"))
        })
        .transpose()
        .map_err(Into::into)
}

fn optional_u64(
    table: &toml::map::Map<String, TomlValue>,
    key: &str,
) -> Result<Option<u64>, Box<dyn Error>> {
    optional_i64(table, key)?
        .map(|value| {
            u64::try_from(value).map_err(|_| format!("{key} must be a non-negative integer"))
        })
        .transpose()
        .map_err(Into::into)
}

fn string_array(
    table: &toml::map::Map<String, TomlValue>,
    key: &str,
    label: &str,
) -> Result<Option<Vec<String>>, Box<dyn Error>> {
    table
        .get(key)
        .map(|value| {
            value
                .as_array()
                .ok_or_else(|| format!("{label} must be an array"))?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| format!("{label} must contain only strings"))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()
        .map_err(Into::into)
}

fn resolve(base: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        normalize(path)
    } else {
        normalize(&base.join(path))
    }
}

fn absolutize(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        normalize(path)
    } else {
        normalize(&base.join(path))
    }
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn find_workspace_root(mut dir: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    loop {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err("failed to find workspace Cargo.toml".into());
        }
    }
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

fn collect_refs(value: &JsonValue) -> Vec<String> {
    let mut refs = Vec::new();
    collect_refs_into(value, &mut refs);
    refs.sort();
    refs.dedup();
    refs
}

fn collect_refs_into(value: &JsonValue, refs: &mut Vec<String>) {
    match value {
        JsonValue::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(JsonValue::as_str) {
                refs.push(reference.to_owned());
            }
            for value in object.values() {
                collect_refs_into(value, refs);
            }
        }
        JsonValue::Array(values) => {
            for value in values {
                collect_refs_into(value, refs);
            }
        }
        _ => {}
    }
}

fn count_key(value: &JsonValue, key: &str) -> usize {
    match value {
        JsonValue::Object(object) => {
            let here = object.contains_key(key) as usize;
            here + object
                .values()
                .map(|value| count_key(value, key))
                .sum::<usize>()
        }
        JsonValue::Array(values) => values.iter().map(|value| count_key(value, key)).sum(),
        _ => 0,
    }
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

#[cfg(test)]
mod tests {
    use super::{relative_child_path, validate_relative_spec_path};
    use std::path::Path;

    #[test]
    fn relative_child_path_uses_manifest_relative_paths() {
        let relative = relative_child_path(
            Path::new("/workspace/graphene/specs/swaplock"),
            Path::new("/workspace/graphene/specs/swaplock/api.database.json"),
        )
        .unwrap();

        assert_eq!(relative, "api.database.json");
    }

    #[test]
    fn spec_paths_must_stay_inside_bundle() {
        assert!(validate_relative_spec_path("api.database.json").is_ok());
        assert!(validate_relative_spec_path("nested/api.database.json").is_ok());
        assert!(validate_relative_spec_path("../api.database.json").is_err());
        assert!(validate_relative_spec_path("/tmp/api.database.json").is_err());
        assert!(validate_relative_spec_path("").is_err());
    }
}
