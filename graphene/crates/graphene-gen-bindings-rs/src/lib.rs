//! Rust binding generation from open-graphene OpenRPC surface specs.
//!
//! This crate emits data-model bindings only: typify-generated schema structs plus
//! custom Graphene `static_variant` enums that preserve the `[index, payload]`
//! JSON wire shape. Transport/runtime SDK layers are intentionally out of scope.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug)]
pub struct ChainBindingJob {
    pub chain: String,
    pub manifest: PathBuf,
    pub crate_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct GenerateOptions {
    pub workspace_root: PathBuf,
    pub jobs: Vec<ChainBindingJob>,
}

pub fn default_options(workspace_root: impl Into<PathBuf>) -> GenerateOptions {
    let workspace_root = workspace_root.into();
    GenerateOptions {
        jobs: vec![
            ChainBindingJob {
                chain: "acta".to_owned(),
                manifest: workspace_root.join("graphene/specs/acta/open-graphene.json"),
                crate_dir: workspace_root.join("graphene-rs/crates/graphene-bindings-acta"),
            },
            ChainBindingJob {
                chain: "swaplock".to_owned(),
                manifest: workspace_root.join("graphene/specs/swaplock/open-graphene.json"),
                crate_dir: workspace_root.join("graphene-rs/crates/graphene-bindings-swaplock"),
            },
        ],
        workspace_root,
    }
}

pub fn generate_all(options: &GenerateOptions) -> Result<()> {
    for job in &options.jobs {
        generate_chain(job)?;
    }
    Ok(())
}

pub fn generate_chain(job: &ChainBindingJob) -> Result<()> {
    let manifest_source = fs::read_to_string(&job.manifest)
        .map_err(|error| format!("failed to read {}: {error}", job.manifest.display()))?;
    let manifest: Value = serde_json::from_str(&manifest_source)
        .map_err(|error| format!("failed to parse {}: {error}", job.manifest.display()))?;
    let manifest_dir = job
        .manifest
        .parent()
        .ok_or_else(|| format!("manifest has no parent: {}", job.manifest.display()))?;
    let surface_paths = surface_spec_paths(&manifest, manifest_dir)?;

    let mut merged_schemas = Map::new();
    let mut surface_specs = Vec::new();
    for surface_path in &surface_paths {
        let source = fs::read_to_string(surface_path)
            .map_err(|error| format!("failed to read {}: {error}", surface_path.display()))?;
        let spec: Value = serde_json::from_str(&source)
            .map_err(|error| format!("failed to parse {}: {error}", surface_path.display()))?;
        let schemas = spec
            .pointer("/components/schemas")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("{} is missing components.schemas", surface_path.display()))?;
        for (name, schema) in schemas {
            if let Some(existing) = merged_schemas.get(name) {
                if existing != schema {
                    return Err(format!(
                        "schema {name} differs across surfaces while generating {}",
                        job.chain
                    )
                    .into());
                }
            } else {
                merged_schemas.insert(name.clone(), schema.clone());
            }
        }
        surface_specs.push((surface_path.clone(), spec));
    }

    let generated_dir = job.crate_dir.join("src/generated");
    fs::create_dir_all(&generated_dir)?;

    let variants = collect_variants(&merged_schemas);
    let variant_names = variants
        .iter()
        .map(|variant| variant.rust_type.clone())
        .collect::<BTreeSet<_>>();

    let schema_doc = typify_schema_doc(&job.chain, &merged_schemas);
    let types_source = typify_types(&schema_doc, &variant_names)?;
    fs::write(generated_dir.join("types.rs"), types_source)?;
    fs::write(
        generated_dir.join("variants.rs"),
        emit_variants(&variants, &job.chain),
    )?;
    fs::write(
        generated_dir.join("metadata.rs"),
        emit_metadata(&variants, &job.chain),
    )?;
    fs::write(
        generated_dir.join("scalar.rs"),
        emit_scalar_extensions(&merged_schemas, &job.chain),
    )?;
    write_rpc_outputs(&generated_dir, &surface_specs, &job.chain)?;

    write_crate_lib(&job.crate_dir, &job.chain)?;
    ensure_crate_dependencies(&job.crate_dir)?;

    println!(
        "generated {} Rust bindings: {} schemas, {} static variants -> {}",
        job.chain,
        merged_schemas.len(),
        variants.len(),
        job.crate_dir.display()
    );
    Ok(())
}

fn surface_spec_paths(manifest: &Value, manifest_dir: &Path) -> Result<Vec<PathBuf>> {
    let surfaces = manifest
        .pointer("/x-open-graphene/surfaces")
        .and_then(Value::as_array)
        .ok_or("manifest is missing x-open-graphene.surfaces")?;
    let mut paths = Vec::new();
    for surface in surfaces {
        let spec = surface
            .get("spec")
            .and_then(Value::as_str)
            .ok_or("surface is missing spec path")?;
        let path = Path::new(spec);
        if path.is_absolute() || spec.contains("..") {
            return Err(format!("invalid surface spec path: {spec}").into());
        }
        paths.push(manifest_dir.join(path));
    }
    Ok(paths)
}

fn typify_schema_doc(chain: &str, schemas: &Map<String, Value>) -> Value {
    let defs = schemas
        .iter()
        .map(|(name, schema)| (name.clone(), rewrite_refs_to_defs(schema)))
        .collect::<Map<_, _>>();
    let mut doc = Map::new();
    doc.insert(
        "$schema".to_owned(),
        Value::String("https://json-schema.org/draft/2020-12/schema".to_owned()),
    );
    doc.insert(
        "title".to_owned(),
        Value::String(format!("{chain}_types_root")),
    );
    doc.insert("$defs".to_owned(), Value::Object(defs));
    Value::Object(doc)
}

fn rewrite_refs_to_defs(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, item)| {
                    if key == "$ref" {
                        if let Some(reference) = item.as_str() {
                            if let Some(name) = reference.strip_prefix("#/components/schemas/") {
                                return (key.clone(), Value::String(format!("#/$defs/{name}")));
                            }
                        }
                    }
                    (key.clone(), rewrite_refs_to_defs(item))
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(rewrite_refs_to_defs).collect()),
        _ => value.clone(),
    }
}

fn typify_types(schema_doc: &Value, strip_names: &BTreeSet<String>) -> Result<String> {
    typify_schema_value(schema_doc, strip_names)
}

pub fn typify_schema_value(schema_doc: &Value, strip_names: &BTreeSet<String>) -> Result<String> {
    let parsed: schemars::schema::RootSchema = serde_json::from_value(schema_doc.clone())?;
    let mut settings = typify::TypeSpaceSettings::default();
    settings.with_struct_builder(true);
    let mut typespace = typify::TypeSpace::new(&settings);
    typespace.add_root_schema(parsed)?;

    let tokens = typespace.to_stream();
    let formatted = match syn::parse2::<syn::File>(tokens.clone()) {
        Ok(file) => prettyplease::unparse(&file),
        Err(_) => tokens.to_string(),
    };
    Ok(strip_named_items(&formatted, strip_names))
}

#[derive(Clone, Debug)]
struct StaticVariant {
    schema_name: String,
    rust_type: String,
    cpp_type: String,
    alternatives: Vec<StaticVariantAlternative>,
}

#[derive(Clone, Debug)]
struct StaticVariantAlternative {
    index: u64,
    rust_variant: String,
    rust_type: String,
    cpp_type: String,
    schema: Option<String>,
    operation_name: Option<String>,
}

fn collect_variants(schemas: &Map<String, Value>) -> Vec<StaticVariant> {
    let mut variants = Vec::new();
    for (schema_name, schema) in schemas {
        let Some(metadata) = schema
            .get("x-graphene-static-variant")
            .and_then(Value::as_object)
        else {
            continue;
        };
        let Some(alternatives_meta) = metadata.get("alternatives").and_then(Value::as_array) else {
            continue;
        };
        let operation_metadata = schema
            .get("x-graphene-operation")
            .and_then(Value::as_object)
            .and_then(|metadata| metadata.get("operations"))
            .and_then(Value::as_array)
            .map(|operations| {
                operations
                    .iter()
                    .filter_map(|operation| {
                        let operation = operation.as_object()?;
                        let index = operation.get("operationId")?.as_u64()?;
                        let name = operation.get("name")?.as_str()?.to_owned();
                        Some((index, name))
                    })
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        let mut used = BTreeSet::new();
        let mut alternatives = Vec::new();
        for alternative in alternatives_meta {
            let Some(alternative) = alternative.as_object() else {
                continue;
            };
            let index = alternative
                .get("index")
                .and_then(Value::as_u64)
                .unwrap_or(alternatives.len() as u64);
            let cpp_type = alternative
                .get("cppType")
                .and_then(Value::as_str)
                .unwrap_or("value")
                .to_owned();
            let schema = alternative.get("schema").and_then(schema_ref_string);
            let rust_type = alternative
                .get("schema")
                .map(schema_fragment_rust_type)
                .unwrap_or_else(|| "serde_json::Value".to_owned());
            alternatives.push(StaticVariantAlternative {
                index,
                rust_variant: variant_ident(&cpp_type, &mut used),
                rust_type,
                cpp_type,
                schema,
                operation_name: operation_metadata.get(&index).cloned(),
            });
        }
        variants.push(StaticVariant {
            schema_name: schema_name.clone(),
            rust_type: snake_to_pascal(schema_name),
            cpp_type: schema
                .get("x-cpp-type")
                .and_then(Value::as_str)
                .unwrap_or(schema_name)
                .to_owned(),
            alternatives,
        });
    }
    variants.sort_by(|left, right| left.schema_name.cmp(&right.schema_name));
    variants
}

fn schema_ref_string(fragment: &Value) -> Option<String> {
    match fragment {
        Value::String(reference) => Some(reference.clone()),
        Value::Object(object) => object
            .get("$ref")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        _ => None,
    }
}

fn schema_fragment_rust_type(fragment: &Value) -> String {
    match fragment {
        Value::String(reference) => ref_rust_type(reference),
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                return ref_rust_type(reference);
            }
            if let Some(one_of) = object.get("oneOf").and_then(Value::as_array) {
                let non_null = one_of
                    .iter()
                    .filter(|item| item.get("type").and_then(Value::as_str) != Some("null"))
                    .collect::<Vec<_>>();
                if non_null.len() == 1 && non_null.len() != one_of.len() {
                    return format!("Option<{}>", schema_fragment_rust_type(non_null[0]));
                }
                return "serde_json::Value".to_owned();
            }
            if let Some(types) = object.get("type").and_then(Value::as_array) {
                let non_null = types
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|ty| *ty != "null")
                    .collect::<Vec<_>>();
                if non_null.len() == 1 && non_null.len() != types.len() {
                    let mut clone = object.clone();
                    clone.insert("type".to_owned(), Value::String(non_null[0].to_owned()));
                    return format!(
                        "Option<{}>",
                        schema_fragment_rust_type(&Value::Object(clone))
                    );
                }
                return "serde_json::Value".to_owned();
            }
            match object.get("type").and_then(Value::as_str) {
                Some("null") => "()".to_owned(),
                Some("boolean") => "bool".to_owned(),
                Some("string") => "String".to_owned(),
                Some("integer") => match object.get("format").and_then(Value::as_str) {
                    Some("int8") => "i8".to_owned(),
                    Some("int16") => "i16".to_owned(),
                    Some("int32") => "i32".to_owned(),
                    Some("uint8") => "u8".to_owned(),
                    Some("uint16") => "u16".to_owned(),
                    Some("uint32") => "u32".to_owned(),
                    Some("uint64") => "u64".to_owned(),
                    _ => "i64".to_owned(),
                },
                Some("number") => "f64".to_owned(),
                Some("array") => {
                    if let Some(prefix_items) = object.get("prefixItems").and_then(Value::as_array)
                    {
                        return format!(
                            "({})",
                            prefix_items
                                .iter()
                                .map(schema_fragment_rust_type)
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                    let item = object.get("items").unwrap_or(&Value::Null);
                    format!("Vec<{}>", schema_fragment_rust_type(item))
                }
                _ => "serde_json::Value".to_owned(),
            }
        }
        _ => "serde_json::Value".to_owned(),
    }
}

fn ref_rust_type(reference: &str) -> String {
    let name = reference.rsplit('/').next().unwrap_or(reference);
    if name == "precomputable_transaction" {
        return "SignedTransaction".to_owned();
    }
    snake_to_pascal(name)
}

fn emit_variants(variants: &[StaticVariant], chain: &str) -> String {
    let mut out = String::new();
    out.push_str("// @generated by graphene-gen-bindings-rs. Do not edit.\n");
    out.push_str(&format!(
        "// Chain: {chain}; Graphene static_variant bindings.\n\n"
    ));
    out.push_str("use serde::de::{Deserialize, Deserializer, Error as _};\n");
    out.push_str("use serde::ser::{Serialize, SerializeTuple, Serializer};\n\n");

    for variant in variants {
        out.push_str(&format!(
            "/// `{}` — {} Graphene static_variant alternatives.\n",
            variant.cpp_type,
            variant.alternatives.len()
        ));
        out.push_str("#[derive(Clone, Debug)]\n");
        out.push_str(&format!("pub enum {} {{\n", variant.rust_type));
        for alternative in &variant.alternatives {
            out.push_str(&format!(
                "    /// Wire index `{}` — C++ `{}`.\n",
                alternative.index, alternative.cpp_type
            ));
            out.push_str(&format!(
                "    {}({}),\n",
                alternative.rust_variant, alternative.rust_type
            ));
        }
        out.push_str("}\n\n");

        out.push_str(&format!("impl Serialize for {} {{\n", variant.rust_type));
        out.push_str("    fn serialize<__S: Serializer>(&self, serializer: __S) -> Result<__S::Ok, __S::Error> {\n");
        out.push_str("        let mut tuple = serializer.serialize_tuple(2)?;\n");
        out.push_str("        match self {\n");
        for alternative in &variant.alternatives {
            out.push_str(&format!(
                "            {}::{}(value) => {{ tuple.serialize_element(&{}u32)?; tuple.serialize_element(value)?; }}\n",
                variant.rust_type, alternative.rust_variant, alternative.index
            ));
        }
        out.push_str("        }\n");
        out.push_str("        tuple.end()\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        out.push_str(&format!(
            "impl<'de> Deserialize<'de> for {} {{\n",
            variant.rust_type
        ));
        out.push_str("    fn deserialize<__D: Deserializer<'de>>(deserializer: __D) -> Result<Self, __D::Error> {\n");
        out.push_str("        let (index, payload): (u32, serde_json::Value) = Deserialize::deserialize(deserializer)?;\n");
        out.push_str("        match index {\n");
        for alternative in &variant.alternatives {
            out.push_str(&format!(
                "            {} => serde_json::from_value(payload).map({}::{}).map_err(__D::Error::custom),\n",
                alternative.index, variant.rust_type, alternative.rust_variant
            ));
        }
        out.push_str(&format!(
            "            other => Err(__D::Error::custom(format!(\"unknown {} variant index: {{}}\", other))),\n",
            variant.rust_type
        ));
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");
    }

    out
}

fn emit_metadata(variants: &[StaticVariant], chain: &str) -> String {
    let operations = variants
        .iter()
        .find(|variant| variant.schema_name == "operation")
        .map(|variant| {
            variant
                .alternatives
                .iter()
                .filter_map(|alternative| {
                    Some((
                        alternative.index,
                        alternative.operation_name.as_deref()?,
                        alternative.cpp_type.as_str(),
                        alternative.rust_type.as_str(),
                        alternative.schema.as_deref().unwrap_or(""),
                    ))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut out = String::new();
    out.push_str("// @generated by graphene-gen-bindings-rs. Do not edit.\n");
    out.push_str(&format!("// Chain: {chain}; operation metadata.\n\n"));
    out.push_str("#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n");
    out.push_str("pub struct OperationMetadata {\n");
    out.push_str("    pub id: u16,\n");
    out.push_str("    pub name: &'static str,\n");
    out.push_str("    pub cpp_type: &'static str,\n");
    out.push_str("    pub rust_type: &'static str,\n");
    out.push_str("    pub schema: &'static str,\n");
    out.push_str("}\n\n");
    out.push_str("pub const OPERATIONS: &[OperationMetadata] = &[\n");
    for (id, name, cpp_type, rust_type, schema) in operations {
        out.push_str(&format!(
            "    OperationMetadata {{ id: {id}, name: {name:?}, cpp_type: {cpp_type:?}, rust_type: {rust_type:?}, schema: {schema:?} }},\n"
        ));
    }
    out.push_str("];\n");
    out
}

fn emit_scalar_extensions(schemas: &Map<String, Value>, chain: &str) -> String {
    let has_int64 = schemas.contains_key("GrapheneInt64");
    let has_time_point_sec = schemas.contains_key("GrapheneTimePointSec");
    let mut out = String::new();
    out.push_str("// @generated by graphene-gen-bindings-rs. Do not edit.\n");
    out.push_str(&format!("// Chain: {chain}; scalar ergonomics.\n\n"));

    if has_time_point_sec {
        out.push_str("use chrono::NaiveDateTime;\n\n");
    }

    if has_int64 {
        out.push_str(
            "impl GrapheneInt64 {\n    pub fn new(value: i64) -> Self {\n        Self::from(value)\n    }\n\n    pub fn as_i64(&self) -> i64 {\n        match self {\n            Self::Variant0(value) => *value,\n            Self::Variant1(value) => value\n                .parse()\n                .expect(\"validated GrapheneInt64 decimal string should parse as i64\"),\n        }\n    }\n}\n\n",
        );
    }

    if has_time_point_sec {
        out.push_str(
            "impl GrapheneTimePointSec {\n    pub const FORMAT: &'static str = \"%Y-%m-%dT%H:%M:%S\";\n\n    pub fn new(value: NaiveDateTime) -> Self {\n        value\n            .format(Self::FORMAT)\n            .to_string()\n            .parse()\n            .expect(\"formatted GrapheneTimePointSec should match generated validation pattern\")\n    }\n\n    pub fn naive_utc(&self) -> NaiveDateTime {\n        NaiveDateTime::parse_from_str(self.as_str(), Self::FORMAT)\n            .expect(\"validated GrapheneTimePointSec should parse as NaiveDateTime\")\n    }\n}\n\nimpl From<GrapheneTimePointSec> for graphene_rpc::GrapheneTimePointSec {\n    fn from(value: GrapheneTimePointSec) -> Self {\n        graphene_rpc::GrapheneTimePointSec::new(value.naive_utc())\n    }\n}\n\nimpl From<&GrapheneTimePointSec> for graphene_rpc::GrapheneTimePointSec {\n    fn from(value: &GrapheneTimePointSec) -> Self {\n        graphene_rpc::GrapheneTimePointSec::new(value.naive_utc())\n    }\n}\n",
        );
    }

    out
}

fn write_crate_lib(crate_dir: &Path, chain: &str) -> Result<()> {
    let title = upper_camel(chain);
    let has_runtime_helpers = crate_dir.join("src/transfer.rs").exists();
    let has_rpc = crate_dir.join("src/generated/rpc.rs").exists();
    let has_broadcast_rpc = crate_dir.join("src/generated/broadcast_rpc.rs").exists();
    let has_scalar = crate_dir.join("src/generated/scalar.rs").exists();
    let scalar_include = if has_scalar {
        "    include!(\"generated/scalar.rs\");\n"
    } else {
        ""
    };

    let source = if has_runtime_helpers && has_rpc && has_broadcast_rpc {
        format!(
            "//! Rust data models and minimal runtime helpers for {title}.\n//!\n//! The generated modules provide schema structs and Graphene `static_variant`\n//! enums. Hand-written modules may provide chain-local transaction, signing,\n//! and broadcast helpers on top of the shared runtime crates.\n\n#![allow(clippy::all)]\n#![allow(dead_code)]\n#![allow(non_camel_case_types)]\n#![allow(non_snake_case)]\n#![allow(unused_imports)]\n#![allow(clippy::large_enum_variant)]\n#![allow(clippy::enum_variant_names)]\n\npub mod generated {{\n    include!(\"generated/types.rs\");\n    include!(\"generated/variants.rs\");\n    include!(\"generated/metadata.rs\");\n{scalar_include}    include!(\"generated/rpc.rs\");\n}}\n\npub mod broadcast {{\n    use super::generated::*;\n    include!(\"generated/broadcast_rpc.rs\");\n}}\n\nmod account;\nmod codec;\nmod dynamic_global_properties;\nmod transaction;\nmod transfer;\n\npub use account::{{lookup_exact_account_id, LookupAccountError}};\npub use dynamic_global_properties::{{\n    subscribe_dynamic_global_properties, SetSubscribeCallbackParams,\n}};\npub use generated::*;\npub use graphene_rpc::database_callbacks::{{\n    set_block_applied_callback, BlockAppliedNotice, BlockAppliedNoticeError,\n    SetBlockAppliedCallbackParams,\n}};\npub use graphene_transaction::broadcast::{{BroadcastResultError, SynchronousBroadcastResult}};\npub use transaction::{{\n    broadcast_signed_transaction, broadcast_signed_transaction_synchronous,\n    broadcast_signed_transaction_synchronous_typed, broadcast_signed_transaction_with_callback,\n    broadcast_signed_transaction_with_callback_typed, sign_transaction,\n    validate_signed_transaction, BroadcastTransactionWithCallbackParams, PreparedTransaction,\n    SignTransactionError, SignedTransactionEnvelope,\n}};\npub use transfer::{{\n    apply_required_fee, build_transfer_operation, fetch_required_fee_for_transfer,\n    prepare_transaction, prepare_transfer_transaction, BuildTransactionError, TransferDraft,\n}};\n"
        )
    } else {
        let generated_includes = if has_rpc && has_broadcast_rpc {
            format!(
                "    include!(\"generated/types.rs\");\n    include!(\"generated/variants.rs\");\n    include!(\"generated/metadata.rs\");\n{scalar_include}    include!(\"generated/rpc.rs\");"
            )
        } else {
            format!(
                "    include!(\"generated/types.rs\");\n    include!(\"generated/variants.rs\");\n    include!(\"generated/metadata.rs\");\n{scalar_include}"
            )
        };
        let broadcast_module = if has_broadcast_rpc {
            "\npub mod broadcast {\n    use super::generated::*;\n    include!(\"generated/broadcast_rpc.rs\");\n}\n"
        } else {
            ""
        };
        format!(
            "//! Generated Rust data models for {title}.\n//!\n//! This crate contains schema structs, Graphene static_variant enums, and typed\n//! OpenRPC parameter structs generated from open-graphene specs. Transport and\n//! runtime SDK layers are intentionally not generated here.\n\n#![allow(clippy::all)]\n#![allow(dead_code)]\n#![allow(non_camel_case_types)]\n#![allow(non_snake_case)]\n#![allow(unused_imports)]\n#![allow(clippy::large_enum_variant)]\n#![allow(clippy::enum_variant_names)]\n\npub mod generated {{\n{generated_includes}\n}}\n{broadcast_module}\npub use generated::*;\n"
        )
    };
    fs::write(crate_dir.join("src/lib.rs"), source)?;
    Ok(())
}

fn ensure_crate_dependencies(crate_dir: &Path) -> Result<()> {
    let cargo_toml = crate_dir.join("Cargo.toml");
    let source = fs::read_to_string(&cargo_toml)?;
    let package = source
        .split("[dependencies]")
        .next()
        .unwrap_or(source.as_str())
        .trim_end();
    let has_runtime_helpers = crate_dir.join("src/transfer.rs").exists();
    let has_rpc = crate_dir.join("src/generated/rpc.rs").exists()
        || crate_dir.join("src/generated/broadcast_rpc.rs").exists();
    let has_scalar = crate_dir.join("src/generated/scalar.rs").exists();
    let dependencies = if has_runtime_helpers {
        "chrono.workspace = true\ngraphene-codec.workspace = true\ngraphene-rpc.workspace = true\ngraphene-signing.workspace = true\ngraphene-transaction.workspace = true\nregress.workspace = true\nserde.workspace = true\nserde_json.workspace = true\n"
    } else if has_rpc || has_scalar {
        "chrono.workspace = true\ngraphene-rpc.workspace = true\nregress.workspace = true\nserde.workspace = true\nserde_json.workspace = true\n"
    } else {
        "regress = \"0.10\"\nserde = { version = \"1\", features = [\"derive\"] }\nserde_json = \"1\"\n"
    };
    let updated = format!("{package}\n\n[dependencies]\n{dependencies}");
    fs::write(cargo_toml, updated)?;
    Ok(())
}

fn write_rpc_outputs(
    generated_dir: &Path,
    surface_specs: &[(PathBuf, Value)],
    chain: &str,
) -> Result<()> {
    let mut database_specs = Vec::new();
    let mut broadcast_specs = Vec::new();
    for (path, spec) in surface_specs {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if file_name.contains("network-broadcast") || file_name.contains("broadcast") {
            broadcast_specs.push((path.clone(), spec.clone()));
        } else {
            database_specs.push((path.clone(), spec.clone()));
        }
    }

    if !database_specs.is_empty() {
        fs::write(
            generated_dir.join("rpc.rs"),
            emit_rpc_module(&database_specs, chain, "database"),
        )?;
    }
    if !broadcast_specs.is_empty() {
        fs::write(
            generated_dir.join("broadcast_rpc.rs"),
            emit_rpc_module(&broadcast_specs, chain, "network-broadcast"),
        )?;
    }
    Ok(())
}

fn emit_rpc_module(specs: &[(PathBuf, Value)], chain: &str, surface: &str) -> String {
    let mut methods = Vec::new();
    let mut schemas = Map::new();
    let mut source_labels = Vec::new();
    for (path, spec) in specs {
        source_labels.push(path.display().to_string());
        if let Some(spec_methods) = spec.get("methods").and_then(Value::as_array) {
            methods.extend(spec_methods.iter().cloned());
        }
        if let Some(spec_schemas) = spec
            .pointer("/components/schemas")
            .and_then(Value::as_object)
        {
            for (name, schema) in spec_schemas {
                schemas
                    .entry(name.clone())
                    .or_insert_with(|| schema.clone());
            }
        }
    }

    let mut out = String::new();
    writeln!(
        out,
        "// @generated by graphene-gen-bindings-rs. Do not edit."
    )
    .unwrap();
    writeln!(
        out,
        "// Chain: {chain}; surface: {surface}; OpenRPC method params."
    )
    .unwrap();
    writeln!(out, "// Source: {}\n", source_labels.join(", ")).unwrap();
    writeln!(out, "use graphene_rpc::OpenRpcParams;\n").unwrap();

    let method_names = methods
        .iter()
        .filter_map(|method| method.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    writeln!(
        out,
        "/// OpenRPC method names generated for this chain surface."
    )
    .unwrap();
    writeln!(out, "pub const OPENRPC_METHODS: &[&str] = &[").unwrap();
    for name in &method_names {
        writeln!(out, "    {name:?},").unwrap();
    }
    writeln!(out, "];\n").unwrap();

    if method_names.contains(&"get_objects") {
        emit_get_object_result(&mut out, &schemas);
    }
    if method_names.contains(&"get_required_fees") {
        emit_required_fee(&mut out);
    }
    if method_names.contains(&"lookup_vote_ids") {
        emit_lookup_vote_id_object(&mut out);
    }

    let mut used_structs = BTreeSet::new();
    for method in methods {
        let Some(name) = method.get("name").and_then(Value::as_str) else {
            continue;
        };
        let struct_name = format!("{}Params", snake_to_pascal(name));
        if !used_structs.insert(struct_name.clone()) {
            continue;
        }
        let params = method
            .get("params")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let result_type = method_result_type(&method);

        emit_doc(
            &mut out,
            method.get("description").and_then(Value::as_str),
            "",
        );
        if params.is_empty() {
            writeln!(out, "#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]").unwrap();
            writeln!(out, "pub struct {struct_name};\n").unwrap();
        } else {
            writeln!(
                out,
                "#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]"
            )
            .unwrap();
            writeln!(out, "pub struct {struct_name} {{").unwrap();
            for param in &params {
                let param_name = param.get("name").and_then(Value::as_str).unwrap_or("param");
                emit_doc(
                    &mut out,
                    param.get("description").and_then(Value::as_str),
                    "    ",
                );
                writeln!(
                    out,
                    "    pub {}: {},",
                    rust_field_name(param_name),
                    schema_fragment_rust_type(param.get("schema").unwrap_or(&Value::Null))
                )
                .unwrap();
            }
            writeln!(out, "}}\n").unwrap();
        }

        writeln!(out, "impl OpenRpcParams for {struct_name} {{").unwrap();
        writeln!(out, "    const METHOD: &'static str = {name:?};").unwrap();
        writeln!(out, "    type Response = {result_type};\n").unwrap();
        writeln!(
            out,
            "    fn into_positional_params(self) -> Vec<serde_json::Value> {{"
        )
        .unwrap();
        if params.is_empty() {
            writeln!(out, "        Vec::new()").unwrap();
        } else {
            writeln!(out, "        vec![").unwrap();
            for param in &params {
                let param_name = param.get("name").and_then(Value::as_str).unwrap_or("param");
                writeln!(
                    out,
                    "            serde_json::json!(self.{}),",
                    rust_field_name(param_name)
                )
                .unwrap();
            }
            writeln!(out, "        ]").unwrap();
        }
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}\n").unwrap();
    }

    out
}

fn method_result_type(method: &Value) -> String {
    match method.get("name").and_then(Value::as_str) {
        Some("get_objects") => return "Vec<GetObjectResult>".to_owned(),
        Some("get_required_fees") => return "Vec<RequiredFee>".to_owned(),
        Some("lookup_vote_ids") => return "Vec<LookupVoteIdObject>".to_owned(),
        _ => {}
    }
    let result = method.get("result").unwrap_or(&Value::Null);
    if matches!(
        result.get("x-cpp-type").and_then(Value::as_str),
        Some("fc::variant" | "variant")
    ) {
        return "serde_json::Value".to_owned();
    }
    schema_fragment_rust_type(result.get("schema").unwrap_or(&Value::Null))
}

fn emit_get_object_result(out: &mut String, schemas: &Map<String, Value>) {
    let mut object_schema_names = schemas
        .iter()
        .filter_map(|(name, schema)| {
            schema
                .get("x-cpp-type")
                .and_then(Value::as_str)
                .is_some_and(|cpp_type| cpp_type.ends_with("_object"))
                .then_some(name.as_str())
        })
        .collect::<Vec<_>>();
    object_schema_names.sort_unstable();

    writeln!(out, "/// Typed object union returned by `get_objects`.").unwrap();
    writeln!(out, "///").unwrap();
    writeln!(
        out,
        "/// Graphene `get_objects` accepts arbitrary object IDs and returns the"
    )
    .unwrap();
    writeln!(
        out,
        "/// corresponding concrete chain object. Missing objects may be returned as"
    )
    .unwrap();
    writeln!(out, "/// JSON `null` by some nodes.").unwrap();
    writeln!(
        out,
        "#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]"
    )
    .unwrap();
    writeln!(out, "#[serde(untagged)]").unwrap();
    writeln!(out, "pub enum GetObjectResult {{").unwrap();
    for schema_name in object_schema_names {
        let rust_name = snake_to_pascal(schema_name);
        writeln!(out, "    {rust_name}({rust_name}),").unwrap();
    }
    writeln!(out, "    Null(()),").unwrap();
    writeln!(out, "}}\n").unwrap();
}

fn emit_required_fee(out: &mut String) {
    writeln!(out, "/// Typed fee result returned by `get_required_fees`.").unwrap();
    writeln!(out, "///").unwrap();
    writeln!(
        out,
        "/// Normal operations return a single `asset` fee. Proposal-create operations"
    )
    .unwrap();
    writeln!(
        out,
        "/// return an FC pair of the proposal fee and recursively nested proposed"
    )
    .unwrap();
    writeln!(out, "/// operation fees: `[fee, [nested_fee, ...]]`.").unwrap();
    writeln!(
        out,
        "#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]"
    )
    .unwrap();
    writeln!(out, "#[serde(untagged)]").unwrap();
    writeln!(out, "pub enum RequiredFee {{").unwrap();
    writeln!(out, "    Asset(Asset),").unwrap();
    writeln!(out, "    ProposalCreate((Asset, Vec<RequiredFee>)),").unwrap();
    writeln!(out, "}}\n").unwrap();
}

fn emit_lookup_vote_id_object(out: &mut String) {
    writeln!(out, "/// Typed object union returned by `lookup_vote_ids`.").unwrap();
    writeln!(out, "///").unwrap();
    writeln!(
        out,
        "/// Graphene returns concrete vote target objects in one heterogeneous array."
    )
    .unwrap();
    writeln!(
        out,
        "/// The object `id` space identifies the concrete shape: committee members"
    )
    .unwrap();
    writeln!(
        out,
        "/// use `1.5.x`, witnesses use `1.6.x`, and workers use the worker object space."
    )
    .unwrap();
    writeln!(
        out,
        "#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]"
    )
    .unwrap();
    writeln!(out, "#[serde(untagged)]").unwrap();
    writeln!(out, "pub enum LookupVoteIdObject {{").unwrap();
    writeln!(out, "    CommitteeMember(CommitteeMemberObject),").unwrap();
    writeln!(out, "    Witness(WitnessObject),").unwrap();
    writeln!(out, "    Worker(WorkerObject),").unwrap();
    writeln!(out, "}}\n").unwrap();
}

fn emit_doc(out: &mut String, text: Option<&str>, indent: &str) {
    let Some(text) = text else { return };
    for line in text.trim().lines() {
        let line = line.trim();
        if line.is_empty() {
            writeln!(out, "{indent}///").unwrap();
        } else {
            writeln!(out, "{indent}/// {line}").unwrap();
        }
    }
}

fn rust_field_name(name: &str) -> String {
    let mut clean = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if clean.is_empty() || clean.starts_with(|ch: char| ch.is_ascii_digit()) {
        clean = format!("param_{clean}");
    }
    if is_rust_keyword(&clean) {
        format!("r#{clean}")
    } else {
        clean
    }
}

fn is_rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
    )
}

fn strip_named_items(source: &str, names: &BTreeSet<String>) -> String {
    let mut file: syn::File = match syn::parse_str(source) {
        Ok(file) => file,
        Err(_) => return source.to_owned(),
    };
    file.items.retain(|item| keep_item(item, names));
    for item in &mut file.items {
        if let syn::Item::Mod(module) = item {
            if let Some((_brace, items)) = &mut module.content {
                items.retain(|item| keep_item(item, names));
            }
        }
    }
    prettyplease::unparse(&file)
}

fn keep_item(item: &syn::Item, names: &BTreeSet<String>) -> bool {
    match item {
        syn::Item::Enum(item) => !names.contains(&item.ident.to_string()),
        syn::Item::Struct(item) => !names.contains(&item.ident.to_string()),
        syn::Item::Impl(item) => {
            !(type_mentions_name(&item.self_ty, names)
                || item
                    .trait_
                    .as_ref()
                    .is_some_and(|(_bang, path, _for)| path_mentions_name(path, names)))
        }
        _ => true,
    }
}

fn type_mentions_name(ty: &syn::Type, names: &BTreeSet<String>) -> bool {
    match ty {
        syn::Type::Path(path) => path_mentions_name(&path.path, names),
        syn::Type::Reference(reference) => type_mentions_name(&reference.elem, names),
        syn::Type::Array(array) => type_mentions_name(&array.elem, names),
        syn::Type::Tuple(tuple) => tuple.elems.iter().any(|ty| type_mentions_name(ty, names)),
        syn::Type::Paren(paren) => type_mentions_name(&paren.elem, names),
        syn::Type::Group(group) => type_mentions_name(&group.elem, names),
        _ => false,
    }
}

fn path_mentions_name(path: &syn::Path, names: &BTreeSet<String>) -> bool {
    path.segments.iter().any(|segment| {
        names.contains(&segment.ident.to_string())
            || match &segment.arguments {
                syn::PathArguments::AngleBracketed(arguments) => arguments.args.iter().any(|arg| {
                    matches!(arg, syn::GenericArgument::Type(ty) if type_mentions_name(ty, names))
                }),
                syn::PathArguments::Parenthesized(arguments) => {
                    arguments.inputs.iter().any(|ty| type_mentions_name(ty, names))
                        || matches!(&arguments.output, syn::ReturnType::Type(_, ty) if type_mentions_name(ty, names))
                }
                syn::PathArguments::None => false,
            }
    })
}

fn variant_ident(cpp_type: &str, used: &mut BTreeSet<String>) -> String {
    let leaf = cpp_type.rsplit("::").next().unwrap_or(cpp_type);
    let clean = leaf
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    let trimmed = trim_variant_suffix(&clean);
    let mut ident = snake_to_pascal(trimmed);
    if ident.is_empty() {
        ident = "V".to_owned();
    }
    if !ident.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
        ident = format!("V{ident}");
    }
    let base = ident.clone();
    let mut index = 2;
    while used.contains(&ident) {
        ident = format!("{base}{index}");
        index += 1;
    }
    used.insert(ident.clone());
    ident
}

fn trim_variant_suffix(value: &str) -> &str {
    for suffix in [
        "_operation",
        "_initializer",
        "_authority",
        "_policy",
        "_argument_type",
        "__fee_params_t",
        "_fee_params_t",
    ] {
        if value.ends_with(suffix) && value.len() > suffix.len() + 1 {
            return &value[..value.len() - suffix.len()];
        }
    }
    value
}

fn snake_to_pascal(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn upper_camel(value: &str) -> String {
    snake_to_pascal(&value.replace('-', "_"))
}
