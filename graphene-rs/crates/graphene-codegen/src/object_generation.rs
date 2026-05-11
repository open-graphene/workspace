use std::fs;
use std::path::Path;

/// Per-chain code-generation configuration loaded from `codegen.toml` in a chain crate.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct CodegenConfig {
    /// Chain metadata and source-core location.
    pub chain: ChainGenerationConfig,
    /// FC-reflected objects whose Rust `Object` structs should be generated for this chain.
    pub objects: Vec<ObjectGeneration>,
    /// Object families that exist in the ID space but have no reflected payload beyond `id`.
    #[serde(default)]
    pub marker_objects: Vec<String>,
}

/// Per-chain source metadata.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct ChainGenerationConfig {
    /// Human-readable chain key used in generator output.
    pub name: String,
    /// Chain core path relative to the `graphene-v2` repository root.
    pub core_path: String,
}

/// One configured FC-reflected object whose Rust `Object` struct should be generated.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct ObjectGeneration {
    /// Object family module name, for example `account_balance`.
    pub family_name: String,
    /// C++ class name without namespace, for example `account_balance_object`.
    pub cpp_class: String,
    /// Header path relative to a chain core root.
    pub header_path: String,
    /// C++ source path containing FC reflection, relative to a chain core root.
    pub source_path: String,
}

/// Load one per-chain code-generation config from TOML.
pub fn load_codegen_config(path: &Path) -> Result<CodegenConfig, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    toml::from_str(&source).map_err(|error| format!("failed to parse {}: {error}", path.display()))
}
