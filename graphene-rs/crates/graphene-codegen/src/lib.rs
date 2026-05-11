//! Code-generation tooling for Graphene-family Rust crates.
//!
//! This crate is reserved for code that reads Graphene/BitShares C++ source metadata and emits
//! Rust source files for chain crates. Runtime protocol primitives and macros used by generated
//! code belong in `graphene-protocol`.

mod object_generation;

pub use object_generation::{
    ChainGenerationConfig, CodegenConfig, ObjectGeneration, load_codegen_config,
};

/// One object-id family discovered from a Graphene `GRAPHENE_DEFINE_IDS(...)` macro.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectFamily {
    /// Graphene object space, for example `1` for protocol ids and `2` for implementation ids.
    pub object_space: u8,
    /// Family type id inside the object space, for example `5` in `2.5.x`.
    pub type_id: u8,
    /// Source family name from the C++ macro, for example `account_balance`.
    pub name: String,
}

/// One C++ field declaration discovered for a reflected Graphene object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CppField {
    /// Field name as used by FC reflection and RPC JSON.
    pub name: String,
    /// C++ source type, normalized but not resolved.
    pub cpp_type: String,
}

/// One Rust field after applying the generator's C++ type mapping rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustField {
    /// Rust field name.
    pub name: String,
    /// Rust type expression to emit.
    pub rust_type: String,
}

/// One `fc::static_variant` entry discovered from a named C++ alias.
///
/// The numeric tag is derived from declaration order. Source comments such as `/* 17 */` are
/// validation hints only, and `// VIRTUAL` metadata is preserved separately from tag assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationVariant {
    /// Declaration-order static-variant tag.
    pub tag: u16,
    /// C++ variant type expression with source comments removed.
    pub cpp_type: String,
    /// Whether the source variant line carries `VIRTUAL` metadata.
    pub is_virtual: bool,
}

/// One FC-reflected C++ type and its reflected field order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReflectedObject {
    /// Fully qualified C++ type name, for example `graphene::chain::account_balance_object`.
    pub cpp_type: String,
    /// Field names in FC reflection order.
    pub fields: Vec<String>,
}

/// One protocol header loaded for operation declaration discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolHeaderSource<'a> {
    /// Header path used in diagnostics and later coverage reports.
    pub path: &'a str,
    /// Header source text.
    pub source: &'a str,
}

/// One reflected operation field with source declaration metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationField {
    /// Field name as listed by FC reflection.
    pub name: String,
    /// C++ source type, normalized but not resolved.
    pub cpp_type: String,
    /// Header where the field declaration was found.
    pub source_file: String,
    /// 1-based source line where the field declaration begins.
    pub source_line: usize,
}

/// One generated operation declaration fact in static-variant order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationDeclaration {
    /// Declaration-order static-variant tag.
    pub tag: u16,
    /// Operation C++ type from the static variant alias.
    pub cpp_type: String,
    /// Operation name without C++ namespaces.
    pub name: String,
    /// Reflected fields in FC reflection order.
    pub fields: Vec<OperationField>,
}

/// A reportable operation declaration parsing failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationDeclarationError {
    /// Declaration-order static-variant tag.
    pub tag: u16,
    /// Operation C++ type from the static variant alias.
    pub cpp_type: String,
    /// Operation name without C++ namespaces.
    pub name: String,
    /// Reflected field involved in the failure, when known.
    pub field_name: Option<String>,
    /// Normalized C++ type involved in the failure, when known.
    pub cpp_type_hint: Option<String>,
    /// Header path involved in the failure, when known.
    pub source_file: Option<String>,
    /// 1-based source line involved in the failure, when known.
    pub source_line: Option<usize>,
    /// Human-readable reason suitable for coverage reports.
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeclaredCppField {
    name: String,
    cpp_type: String,
    source_file: String,
    source_line: usize,
}

/// Parse all `GRAPHENE_DEFINE_IDS(...)` object families from C++ source text.
///
/// This is the first generator boundary: it extracts only facts needed by
/// `graphene_protocol::define_object_id_type!`, not Rust source text yet.
///
/// The parser intentionally supports the source shape used by Graphene core:
///
/// ```text
/// GRAPHENE_DEFINE_IDS(protocol, protocol_ids, /* prefix */, /* 1.2.x */ (account))
/// GRAPHENE_DEFINE_IDS(chain, implementation_ids, impl_, /* 2.5.x */ (account_balance))
/// ```
pub fn parse_object_families(source: &str) -> Result<Vec<ObjectFamily>, String> {
    let mut families = Vec::new();
    let mut remaining = source;

    while let Some(macro_start) = remaining.find("GRAPHENE_DEFINE_IDS") {
        remaining = &remaining[macro_start + "GRAPHENE_DEFINE_IDS".len()..];

        let Some(open_paren) = remaining.find('(') else {
            break;
        };

        let after_open = &remaining[open_paren + 1..];
        let Some((macro_body, consumed)) = take_balanced_parentheses_body(after_open) else {
            break;
        };

        families.extend(parse_define_ids_body(macro_body)?);
        remaining = &after_open[consumed..];
    }

    Ok(families)
}

/// Render the Rust module source for one generated object-id family.
///
/// The module source intentionally contains only the macro invocation. The family name, object
/// space, and type id are all emitted explicitly so generated Rust preserves the source facts from
/// `GRAPHENE_DEFINE_IDS(...)` without relying on module-name inference.
pub fn render_object_id_module(family: &ObjectFamily) -> String {
    let mut output = String::new();
    output.push_str("// @generated by graphene-codegen; do not edit by hand.\n");
    output.push_str(&format!(
        "// Source family: {} ({}.{}.x)\n\n",
        family.name, family.object_space, family.type_id
    ));

    output.push_str("graphene_protocol::define_object_id_type! {\n");
    output.push_str(&format!("    name: \"{}\",\n", family.name));
    output.push_str(&format!("    object_space: {},\n", family.object_space));
    output.push_str(&format!("    type_id: {},\n", family.type_id));
    output.push_str("}\n");

    output
}

/// Render `src/types/mod.rs` for generated object-id family modules.
pub fn render_types_mod(families: &[ObjectFamily]) -> String {
    let mut output = String::new();
    output.push_str("// @generated by graphene-codegen; do not edit by hand.\n\n");

    let mut family_names = families
        .iter()
        .map(|family| family.name.as_str())
        .collect::<Vec<_>>();
    family_names.sort_unstable();

    for family_name in family_names {
        output.push_str(&format!("pub mod {family_name};\n"));
    }

    output
}

/// Render a data-only Rust module containing static-variant operation tag facts.
///
/// The generated table intentionally depends only on primitive Rust types. It does not reference
/// runtime operation wrappers, so code generation can expose source-derived tag facts without
/// changing operation deserialization semantics.
pub fn render_operation_variants_module(chain_name: &str, variants: &[OperationVariant]) -> String {
    let const_name = operation_variants_const_name(chain_name);
    let mut output = String::new();
    output.push_str("// @generated by graphene-codegen; do not edit by hand.\n");
    output.push_str(&format!(
        "// Source chain: {chain_name} operation static_variant tags\n\n"
    ));
    output.push_str("#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n");
    output.push_str("pub struct OperationVariantSpec {\n");
    output.push_str("    pub tag: u16,\n");
    output.push_str("    pub cpp_type: &'static str,\n");
    output.push_str("    pub is_virtual: bool,\n");
    output.push_str("}\n\n");
    output.push_str(&format!(
        "pub const {const_name}: &[OperationVariantSpec] = &[\n"
    ));

    for variant in variants {
        output.push_str(&format!(
            "    OperationVariantSpec {{ tag: {}, cpp_type: {:?}, is_virtual: {} }},\n",
            variant.tag, variant.cpp_type, variant.is_virtual
        ));
    }

    output.push_str("];\n");
    output
}

fn operation_variants_const_name(chain_name: &str) -> String {
    let mut const_name = String::new();

    for character in chain_name.chars() {
        if character.is_ascii_alphanumeric() {
            const_name.push(character.to_ascii_uppercase());
        } else if !const_name.ends_with('_') {
            const_name.push('_');
        }
    }

    let const_name = const_name.trim_matches('_');
    if const_name.is_empty() {
        "OPERATION_VARIANTS".to_owned()
    } else {
        format!("{const_name}_OPERATION_VARIANTS")
    }
}

/// Parse the BitShares `using operation = fc::static_variant<...>` alias from C++ source text.
pub fn parse_operation_variants(source: &str) -> Result<Vec<OperationVariant>, String> {
    parse_static_variant_alias(source, "operation")
}

/// Parse a named C++ `using <alias> = fc::static_variant<...>` alias.
///
/// This parser intentionally extracts generator facts rather than attempting to be a general C++
/// parser. It performs a linear scan, splits only on top-level commas, ignores delimiters inside
/// comments/parentheses/template arguments, and validates optional `/* N */` comments against the
/// declaration-order tag it computes.
pub fn parse_static_variant_alias(
    source: &str,
    alias: &str,
) -> Result<Vec<OperationVariant>, String> {
    let variant_body = static_variant_alias_body(source, alias)?;
    let raw_items = split_static_variant_items(variant_body)
        .map_err(|error| format!("could not parse static_variant alias {alias}: {error}"))?;
    let mut variants = Vec::new();

    for raw_item in raw_items {
        let cpp_type = strip_cpp_comments(raw_item)
            .trim()
            .trim_end_matches(',')
            .trim()
            .to_owned();
        if cpp_type.is_empty() {
            continue;
        }

        let tag = u16::try_from(variants.len()).map_err(|_| {
            format!(
                "static_variant alias {alias} has more than {} entries; tag assignment would overflow u16",
                u16::MAX
            )
        })?;

        if let Some(comment_tag) = parse_variant_tag_comment(raw_item) {
            if comment_tag != tag {
                return Err(format!(
                    "static_variant alias {alias} variant {cpp_type}: comment tag {comment_tag} disagrees with declaration order {tag}"
                ));
            }
        }

        variants.push(OperationVariant {
            tag,
            cpp_type,
            is_virtual: raw_item.contains("VIRTUAL"),
        });
    }

    if variants.is_empty() {
        return Err(format!("static_variant alias {alias} has no variants"));
    }

    Ok(variants)
}

/// Parse C++ field declarations for `class_name`, keeping only fields listed by FC reflection.
///
/// This is intentionally narrower than a general C++ parser. It is the first struct-generation
/// boundary: FC reflection decides which field names matter, and this function only recovers their
/// source types from the class declaration.
pub fn parse_reflected_class_fields(
    header_source: &str,
    class_name: &str,
    reflected_fields: &[&str],
) -> Result<Vec<CppField>, String> {
    let body = class_or_struct_body(header_source, class_name)
        .ok_or_else(|| format!("could not find C++ class or struct body for {class_name}"))?;
    let declarations = parse_top_level_declarations(body.body, "<memory>", body.start_line);
    let mut fields = Vec::new();

    for reflected_field in reflected_fields {
        let Some(cpp_type) = declarations
            .iter()
            .find(|field| field.name == *reflected_field)
            .map(|field| field.cpp_type.clone())
        else {
            return Err(format!(
                "could not find reflected field {class_name}::{reflected_field} in class declaration"
            ));
        };
        fields.push(CppField {
            name: (*reflected_field).to_owned(),
            cpp_type,
        });
    }

    Ok(fields)
}

/// Parse reflected operation declarations from protocol headers in static-variant order.
///
/// The function performs one linear declaration/reflection pass over each header and then joins the
/// facts by operation type. Missing reflections, declarations, or reflected fields are returned as
/// reportable errors instead of being silently dropped.
pub fn parse_operation_declarations(
    variants: &[OperationVariant],
    headers: &[ProtocolHeaderSource<'_>],
) -> Result<Vec<OperationDeclaration>, Vec<OperationDeclarationError>> {
    let mut reflected_by_name = Vec::<(String, ReflectedObject)>::new();
    let mut declarations_by_name = Vec::<(String, String, usize, Vec<DeclaredCppField>)>::new();
    let mut errors = Vec::new();

    for header in headers {
        match parse_reflected_objects(header.source) {
            Ok(reflected_objects) => {
                reflected_by_name.extend(
                    reflected_objects
                        .into_iter()
                        .map(|object| (unqualified_cpp_name(&object.cpp_type).to_owned(), object)),
                );
            }
            Err(error) => errors.push(OperationDeclarationError {
                tag: 0,
                cpp_type: "<unknown>".to_owned(),
                name: "<unknown>".to_owned(),
                field_name: None,
                cpp_type_hint: None,
                source_file: Some(header.path.to_owned()),
                source_line: None,
                reason: error,
            }),
        }

        for variant in variants {
            let name = unqualified_cpp_name(&variant.cpp_type);
            if declarations_by_name
                .iter()
                .any(|(declared_name, _, _, _)| declared_name == name)
            {
                continue;
            }
            if let Some(body) = class_or_struct_body(header.source, name) {
                declarations_by_name.push((
                    name.to_owned(),
                    header.path.to_owned(),
                    body.start_line,
                    parse_top_level_declarations(body.body, header.path, body.start_line),
                ));
            }
        }
    }

    let mut declarations = Vec::new();
    for variant in variants {
        let name = unqualified_cpp_name(&variant.cpp_type).to_owned();
        let Some((_, reflected)) = reflected_by_name
            .iter()
            .find(|(reflected_name, _)| reflected_name == &name)
        else {
            errors.push(OperationDeclarationError {
                tag: variant.tag,
                cpp_type: variant.cpp_type.clone(),
                name,
                field_name: None,
                cpp_type_hint: None,
                source_file: None,
                source_line: None,
                reason: "missing FC_REFLECT entry for operation".to_owned(),
            });
            continue;
        };

        let Some((_, declaration_file, declaration_line, declared_fields)) = declarations_by_name
            .iter()
            .find(|(declared_name, _, _, _)| declared_name == &name)
        else {
            errors.push(OperationDeclarationError {
                tag: variant.tag,
                cpp_type: variant.cpp_type.clone(),
                name,
                field_name: None,
                cpp_type_hint: None,
                source_file: None,
                source_line: None,
                reason: "missing class or struct declaration for reflected operation".to_owned(),
            });
            continue;
        };

        let mut fields = Vec::new();
        for reflected_field in &reflected.fields {
            let Some(declared_field) = declared_fields
                .iter()
                .find(|field| field.name == *reflected_field)
            else {
                errors.push(OperationDeclarationError {
                    tag: variant.tag,
                    cpp_type: variant.cpp_type.clone(),
                    name: name.clone(),
                    field_name: Some(reflected_field.clone()),
                    cpp_type_hint: None,
                    source_file: Some(declaration_file.clone()),
                    source_line: Some(*declaration_line),
                    reason: "reflected field is missing from operation declaration".to_owned(),
                });
                continue;
            };
            fields.push(OperationField {
                name: declared_field.name.clone(),
                cpp_type: declared_field.cpp_type.clone(),
                source_file: declared_field.source_file.clone(),
                source_line: declared_field.source_line,
            });
        }

        declarations.push(OperationDeclaration {
            tag: variant.tag,
            cpp_type: variant.cpp_type.clone(),
            name,
            fields,
        });
    }

    if errors.is_empty() {
        Ok(declarations)
    } else {
        Err(errors)
    }
}

/// Classification for one reflected operation field after applying operation-specific mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationFieldClassification {
    /// The field can be emitted as a typed protocol-safe Rust field.
    Typed { rust_type: String },
    /// The field is deliberately preserved as a raw protocol fallback for approved payload shapes.
    ApprovedRawFallback { rust_type: String, reason: String },
    /// The field has no safe mapping and must prevent emitting the containing operation struct.
    Unsupported { reason: String },
}

/// One row in the operation model coverage report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationModelReportRow {
    pub operation: String,
    pub tag: u16,
    pub field: String,
    pub cpp_type: String,
    pub source_file: String,
    pub source_line: usize,
    pub classification: String,
    pub reason: String,
}

/// Rendered operation structs plus coverage metadata for skipped/raw fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationStructRender {
    pub source: String,
    pub report_rows: Vec<OperationModelReportRow>,
    pub generated_struct_count: usize,
    pub unsupported_count: usize,
    pub raw_fallback_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderedOperationVariant {
    tag: u16,
    enum_variant_name: String,
    struct_name: String,
}

/// Map a Graphene C++ operation field into a protocol-safe Rust type classification.
///
/// This mapper intentionally does not call [`map_cpp_type_to_rust`]. Operation payload structs are
/// chain-independent protocol models, so all protocol object IDs collapse to
/// `graphene_protocol::ObjectId` instead of chain crate `crate::types::*::Id` modules.
pub fn map_operation_field(field: &OperationField) -> OperationFieldClassification {
    map_operation_field_for_operation("", field)
}

/// Map a Graphene C++ operation field into a protocol-safe Rust type classification with
/// operation context for deliberately approved raw fallback payload shapes.
pub fn map_operation_field_for_operation(
    operation_name: &str,
    field: &OperationField,
) -> OperationFieldClassification {
    let normalized = normalize_cpp_type(&field.cpp_type);

    if is_approved_operation_raw_fallback(operation_name, &field.name, &normalized) {
        return OperationFieldClassification::ApprovedRawFallback {
            rust_type: approved_operation_raw_fallback_type(&field.name, &normalized),
            reason: approved_operation_raw_fallback_reason(&normalized),
        };
    }

    if normalized.contains("future_extensions")
        || normalized == "extension"
        || normalized.starts_with("extension<")
    {
        return OperationFieldClassification::Unsupported {
            reason:
                "raw fallback is restricted to approved extension/static-variant payload fields"
                    .to_owned(),
        };
    }

    match map_operation_cpp_type_to_rust(&normalized) {
        Some(rust_type) => OperationFieldClassification::Typed { rust_type },
        None => OperationFieldClassification::Unsupported {
            reason: format!("no protocol-safe operation mapping for C++ type {normalized}"),
        },
    }
}

fn map_operation_cpp_type_to_rust(cpp_type: &str) -> Option<String> {
    let normalized = normalize_cpp_type(cpp_type);

    match normalized.as_str() {
        "bool" => return Some("bool".to_owned()),
        "char" | "uint8_t" => return Some("u8".to_owned()),
        "uint16_t" => return Some("u16".to_owned()),
        "uint32_t" | "unsigned_int" => return Some("u32".to_owned()),
        "uint64_t" => return Some("u64".to_owned()),
        "int64_t" | "share_type" => return Some("i64".to_owned()),
        "fc::uint128_t" | "uint128_t" => return Some("u128".to_owned()),
        "string" | "std::string" => return Some("String".to_owned()),
        "time_point_sec" => return Some("String".to_owned()),
        "public_key_type" => return Some("String".to_owned()),
        "vote_id_type" => return Some("String".to_owned()),
        "block_id_type" => return Some("String".to_owned()),
        "chain_id_type" => return Some("String".to_owned()),
        "transaction_id_type" => return Some("String".to_owned()),
        "blind_factor_type" => return Some("graphene_protocol::BlindFactor".to_owned()),
        "blind_input" => return Some("graphene_protocol::BlindInput".to_owned()),
        "blind_output" => return Some("graphene_protocol::BlindOutput".to_owned()),
        "range_proof_type" => return Some("graphene_protocol::RangeProof".to_owned()),
        "commitment_type" | "fc::ecc::commitment_type" => {
            return Some("graphene_protocol::Commitment".to_owned());
        }
        "htlc_hash" => return Some("graphene_protocol::HtlcHash".to_owned()),
        "chain_parameters" => return Some("graphene_protocol::ChainParameters".to_owned()),
        "vesting_policy_initializer" => {
            return Some("graphene_protocol::VestingPolicyInitializer".to_owned());
        }
        "worker_initializer" => return Some("graphene_protocol::WorkerInitializer".to_owned()),
        "predicate" => {
            return Some(
                "graphene_protocol::Predicate<graphene_protocol::ObjectId, graphene_protocol::ObjectId>"
                    .to_owned(),
            );
        }
        "asset" => {
            return Some("graphene_protocol::Asset<graphene_protocol::ObjectId>".to_owned());
        }
        "price" => {
            return Some("graphene_protocol::Price<graphene_protocol::ObjectId>".to_owned());
        }
        "authority" => {
            return Some("graphene_protocol::Authority<graphene_protocol::ObjectId>".to_owned());
        }
        "account_options" => {
            return Some(
                "graphene_protocol::AccountOptions<graphene_protocol::ObjectId>".to_owned(),
            );
        }
        "asset_options" => {
            return Some(
                "graphene_protocol::AssetOptions<graphene_protocol::ObjectId, graphene_protocol::ObjectId>"
                    .to_owned(),
            );
        }
        "bitasset_options" => {
            return Some(
                "graphene_protocol::BitAssetOptions<graphene_protocol::ObjectId>".to_owned(),
            );
        }
        "price_feed" => {
            return Some("graphene_protocol::PriceFeed<graphene_protocol::ObjectId>".to_owned());
        }
        "limit_order_auto_action" => {
            return Some(
                "graphene_protocol::LimitOrderAutoAction<graphene_protocol::ObjectId>".to_owned(),
            );
        }
        "memo_data" => return Some("graphene_protocol::MemoData".to_owned()),
        "restriction" => return Some("graphene_protocol::Restriction".to_owned()),
        "op_wrapper" => return Some("OperationWrapper".to_owned()),
        "operation" | "operation_result" => {
            return Some("graphene_protocol::RestrictionArgument".to_owned());
        }
        _ => {}
    }

    if normalized == "object_id_type" || normalized.ends_with("_id_type") {
        return Some("graphene_protocol::ObjectId".to_owned());
    }

    for template in ["optional", "fc::optional"] {
        if let Some(inner) = template_argument(&normalized, template) {
            return map_operation_cpp_type_to_rust(inner)
                .map(|rust_type| format!("Option<{rust_type}>"));
        }
    }

    if let Some(inner) = template_argument(&normalized, "pair") {
        let (first, second) = split_template_pair(inner)?;
        let first = map_operation_cpp_type_to_rust(first)?;
        let second = map_operation_cpp_type_to_rust(second)?;
        return Some(format!("({first}, {second})"));
    }

    if let Some(inner) = template_argument(&normalized, "flat_map") {
        let (key, value) = split_template_pair(inner)?;
        let key = map_operation_cpp_type_to_rust(key)?;
        let value = map_operation_cpp_type_to_rust(value)?;
        return Some(format!("Vec<({key}, {value})>"));
    }

    for container in ["flat_set", "set", "vector", "std::vector"] {
        if let Some(inner) = template_argument(&normalized, container) {
            return map_operation_cpp_type_to_rust(inner)
                .map(|rust_type| format!("Vec<{rust_type}>"));
        }
    }

    None
}

fn is_approved_operation_raw_fallback(
    operation_name: &str,
    field_name: &str,
    cpp_type: &str,
) -> bool {
    (field_name == "extensions" && cpp_type == "extensions_type")
        || is_approved_operation_extension_fallback(operation_name, field_name, cpp_type)
}

fn is_approved_operation_extension_fallback(
    operation_name: &str,
    field_name: &str,
    cpp_type: &str,
) -> bool {
    field_name == "extensions"
        && matches!(
            (operation_name, cpp_type),
            ("account_create_operation", "extension<ext>")
                | ("account_update_operation", "extension<ext>")
                | ("asset_update_operation", "extension<ext>")
                | ("asset_publish_feed_operation", "extension<ext>")
                | (
                    "asset_claim_fees_operation",
                    "extension<additional_options_type>"
                )
                | (
                    "htlc_create_operation",
                    "extension<additional_options_type>"
                )
                | ("credit_offer_accept_operation", "extension<ext>")
        )
}

fn approved_operation_raw_fallback_type(field_name: &str, cpp_type: &str) -> String {
    if field_name == "extensions" && cpp_type == "extensions_type" {
        "Vec<graphene_protocol::RestrictionArgument>".to_owned()
    } else {
        "graphene_protocol::RestrictionArgument".to_owned()
    }
}

fn approved_operation_raw_fallback_reason(cpp_type: &str) -> String {
    if cpp_type == "extension<ext>" {
        "approved raw fallback for operation-scoped extension<ext> payload".to_owned()
    } else {
        "approved raw fallback for extension/static-variant payload".to_owned()
    }
}

/// Render generated BitShares operation payload structs and collect coverage report rows.
pub fn render_operation_structs_module(
    declarations: &[OperationDeclaration],
) -> OperationStructRender {
    let mut sorted_declarations = declarations.iter().collect::<Vec<_>>();
    sorted_declarations.sort_by_key(|declaration| declaration.tag);

    let mut source = String::new();
    source.push_str("// @generated by graphene-codegen; do not edit by hand.\n\n");

    let mut report_rows = Vec::new();
    let mut rendered_variants = Vec::new();
    let mut generated_struct_count = 0usize;
    let mut unsupported_count = 0usize;
    let mut raw_fallback_count = 0usize;

    for declaration in sorted_declarations {
        let mut rendered_fields = Vec::<RustField>::new();
        let mut has_unsupported = false;

        for field in &declaration.fields {
            match map_operation_field_for_operation(&declaration.name, field) {
                OperationFieldClassification::Typed { rust_type } => {
                    rendered_fields.push(RustField {
                        name: field.name.clone(),
                        rust_type,
                    });
                }
                OperationFieldClassification::ApprovedRawFallback { rust_type, reason } => {
                    raw_fallback_count += 1;
                    report_rows.push(operation_report_row(
                        declaration,
                        field,
                        "approved_raw_fallback",
                        reason.clone(),
                    ));
                    rendered_fields.push(RustField {
                        name: field.name.clone(),
                        rust_type,
                    });
                }
                OperationFieldClassification::Unsupported { reason } => {
                    unsupported_count += 1;
                    has_unsupported = true;
                    report_rows.push(operation_report_row(
                        declaration,
                        field,
                        "unsupported",
                        reason,
                    ));
                }
            }
        }

        if has_unsupported {
            continue;
        }

        generated_struct_count += 1;
        let struct_name = operation_rust_struct_name(&declaration.name);
        rendered_variants.push(RenderedOperationVariant {
            tag: declaration.tag,
            enum_variant_name: operation_enum_variant_name(&struct_name),
            struct_name,
        });
        source.push_str(&render_operation_struct(declaration, &rendered_fields));
        source.push('\n');
    }

    if generated_struct_count > 0 {
        source.pop();
    }

    if !source.ends_with("\n\n") {
        source.push('\n');
    }
    source.push_str(&render_operation_wrapper_struct());
    source.push('\n');
    source.push_str(&render_operation_enum(&rendered_variants));

    OperationStructRender {
        source,
        report_rows,
        generated_struct_count,
        unsupported_count,
        raw_fallback_count,
    }
}

fn render_operation_wrapper_struct() -> String {
    concat!(
        "#[derive(Clone, Debug, PartialEq, serde::Deserialize)]\n",
        "pub struct OperationWrapper {\n",
        "    pub op: Operation,\n",
        "}\n"
    )
    .to_owned()
}

fn render_operation_enum(variants: &[RenderedOperationVariant]) -> String {
    let mut output = String::new();
    output.push_str("#[derive(Clone, Debug, PartialEq)]\n");
    output.push_str("pub enum Operation {\n");
    for variant in variants {
        output.push_str(&format!(
            "    {}({}),\n",
            variant.enum_variant_name, variant.struct_name
        ));
    }
    output.push_str("    Unsupported {\n");
    output.push_str("        tag: u16,\n");
    output.push_str("        payload: serde_json::Value,\n");
    output.push_str("    },\n");
    output.push_str("}\n\n");

    output.push_str("impl Operation {\n");
    output.push_str("    pub fn tag(&self) -> u16 {\n");
    output.push_str("        match self {\n");
    for variant in variants {
        output.push_str(&format!(
            "            Self::{}(_) => {},\n",
            variant.enum_variant_name, variant.tag
        ));
    }
    output.push_str("            Self::Unsupported { tag, .. } => *tag,\n");
    output.push_str("        }\n");
    output.push_str("    }\n\n");
    output.push_str("    pub fn is_typed(&self) -> bool {\n");
    output.push_str("        !matches!(self, Self::Unsupported { .. })\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    output.push_str("impl<'de> serde::Deserialize<'de> for Operation {\n");
    output.push_str(
        "    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\n    where\n        D: serde::Deserializer<'de>,\n    {\n",
    );
    output.push_str("        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;\n");
    output.push_str("        let serde_json::Value::Array(mut elements) = value else {\n");
    output.push_str("            return Err(serde::de::Error::custom(\n");
    output.push_str("                \"operation static_variant must be a two-element array\",\n");
    output.push_str("            ));\n");
    output.push_str("        };\n");
    output.push_str("        if elements.len() != 2 {\n");
    output.push_str("            return Err(serde::de::Error::custom(\n");
    output.push_str("                \"operation static_variant must be a two-element array\",\n");
    output.push_str("            ));\n");
    output.push_str("        }\n");
    output.push_str("        let payload = elements.pop().expect(\"length checked\");\n");
    output.push_str("        let tag_value = elements.pop().expect(\"length checked\");\n");
    output.push_str("        let tag_u64 = tag_value.as_u64().ok_or_else(|| {\n");
    output.push_str(
        "            serde::de::Error::custom(\"operation static_variant tag must be an unsigned integer\")\n        })?;\n",
    );
    output.push_str("        let tag = u16::try_from(tag_u64).map_err(|_| {\n");
    output.push_str(
        "            serde::de::Error::custom(\"operation static_variant tag must fit in u16\")\n        })?;\n\n",
    );
    output.push_str("        match tag {\n");
    for variant in variants {
        output.push_str(&format!(
            "            {} => serde_json::from_value(payload)\n                .map(Self::{})\n                .map_err(serde::de::Error::custom),\n",
            variant.tag, variant.enum_variant_name
        ));
    }
    output.push_str("            _ => Ok(Self::Unsupported { tag, payload }),\n");
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n");
    output
}

fn operation_enum_variant_name(struct_name: &str) -> String {
    struct_name
        .strip_suffix("Operation")
        .unwrap_or(struct_name)
        .to_owned()
}

fn operation_report_row(
    declaration: &OperationDeclaration,
    field: &OperationField,
    classification: &str,
    reason: String,
) -> OperationModelReportRow {
    OperationModelReportRow {
        operation: declaration.name.clone(),
        tag: declaration.tag,
        field: field.name.clone(),
        cpp_type: normalize_cpp_type(&field.cpp_type),
        source_file: field.source_file.clone(),
        source_line: field.source_line,
        classification: classification.to_owned(),
        reason,
    }
}

fn render_operation_struct(declaration: &OperationDeclaration, fields: &[RustField]) -> String {
    let mut output = String::new();
    output.push_str("#[derive(Clone, Debug, PartialEq, serde::Deserialize)]\n");
    output.push_str(&format!(
        "pub struct {}",
        operation_rust_struct_name(&declaration.name)
    ));

    if fields.is_empty() {
        output.push_str(" {}\n");
        return output;
    }

    output.push_str(" {\n");
    for field in fields {
        if let Some(deserializer) = numeric_deserializer(&field.rust_type) {
            output.push_str(&format!(
                "    #[serde(deserialize_with = \"graphene_protocol::{deserializer}\")]\n"
            ));
        }
        output.push_str(&render_operation_struct_field(field));
    }
    output.push_str("}\n");
    output
}

fn render_operation_struct_field(field: &RustField) -> String {
    let declaration_length =
        "    pub ".len() + field.name.len() + ": ".len() + field.rust_type.len() + ",".len();
    if declaration_length <= 100 {
        return format!("    pub {}: {},\n", field.name, field.rust_type);
    }

    let formatted_type = format_long_rust_type(&field.rust_type, "    ");
    format!("    pub {}: {formatted_type},\n", field.name)
}

fn format_long_rust_type(rust_type: &str, indent: &str) -> String {
    if let Some(inner) = rust_type
        .strip_prefix("Option<")
        .and_then(|inner| inner.strip_suffix('>'))
    {
        let child_indent = format!("{indent}    ");
        let inner = format_long_rust_type(inner, &child_indent);
        return format!("Option<\n{child_indent}{inner},\n{indent}>");
    }

    if let Some(inner) = rust_type
        .strip_prefix("Vec<(")
        .and_then(|inner| inner.strip_suffix(")>"))
    {
        if let Some((first, second)) = split_template_pair(inner) {
            let child_indent = format!("{indent}    ");
            return format!("Vec<(\n{child_indent}{first},\n{child_indent}{second},\n{indent})>");
        }
    }

    rust_type.to_owned()
}

fn operation_rust_struct_name(operation_name: &str) -> String {
    operation_name
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => {
                    let mut segment = String::new();
                    segment.push(first.to_ascii_uppercase());
                    segment.extend(characters.map(|character| character.to_ascii_lowercase()));
                    segment
                }
                None => String::new(),
            }
        })
        .collect::<String>()
}

/// Render the operation model coverage report consumed by future generator diagnostics.
pub fn render_operation_model_skips_report(
    total_operations: usize,
    generated_struct_count: usize,
    rows: &[OperationModelReportRow],
) -> String {
    let unsupported_count = rows
        .iter()
        .filter(|row| row.classification == "unsupported")
        .count();
    let raw_fallback_count = rows
        .iter()
        .filter(|row| row.classification == "approved_raw_fallback")
        .count();

    let mut output = String::new();
    output.push_str("# Operation model coverage report\n\n");
    output.push_str("@generated by graphene-codegen; do not edit by hand.\n\n");
    output.push_str(&format!("- Total operations: {total_operations}\n"));
    output.push_str(&format!("- Generated structs: {generated_struct_count}\n"));
    output.push_str(&format!(
        "- Skipped/unsupported rows: {unsupported_count}\n"
    ));
    output.push_str(&format!(
        "- Approved raw fallback rows: {raw_fallback_count}\n\n"
    ));
    output.push_str(
        "| Operation | Tag | Field | Normalized C++ type | Source | Classification | Reason |\n",
    );
    output.push_str("|---|---:|---|---|---|---|---|\n");

    for row in rows {
        output.push_str(&format!(
            "| {} | {} | {} | `{}` | {}:{} | {} | {} |\n",
            escape_markdown_table_cell(&row.operation),
            row.tag,
            escape_markdown_table_cell(&row.field),
            row.cpp_type.replace('`', "\\`"),
            escape_markdown_table_cell(&row.source_file),
            row.source_line,
            escape_markdown_table_cell(&row.classification),
            escape_markdown_table_cell(&row.reason),
        ));
    }

    output
}

fn escape_markdown_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

/// Map a Graphene C++ field type into the Rust type expression used by generated structs.
pub fn map_cpp_type_to_rust(cpp_type: &str) -> Option<String> {
    let normalized = normalize_cpp_type(cpp_type);

    match normalized.as_str() {
        "string" => return Some("String".to_owned()),
        "account_options" => {
            return Some("graphene_protocol::AccountOptions<crate::types::account::Id>".to_owned());
        }
        "address" => return Some("String".to_owned()),
        "asset" => return Some("graphene_protocol::Asset<crate::types::asset::Id>".to_owned()),
        "asset_options" => {
            return Some(
                "graphene_protocol::AssetOptions<crate::types::account::Id, crate::types::asset::Id>"
                    .to_owned(),
            );
        }
        "bitasset_options" => {
            return Some("graphene_protocol::BitAssetOptions<crate::types::asset::Id>".to_owned());
        }
        "budget_record" => return Some("graphene_protocol::BudgetRecord".to_owned()),
        "price" => return Some("graphene_protocol::Price<crate::types::asset::Id>".to_owned()),
        "price_feed_with_icr" => {
            return Some("graphene_protocol::PriceFeedWithIcr<crate::types::asset::Id>".to_owned());
        }
        "bool" => return Some("bool".to_owned()),
        "uint8_t" => return Some("u8".to_owned()),
        "uint16_t" => return Some("u16".to_owned()),
        "uint32_t" => return Some("u32".to_owned()),
        "uint64_t" | "unsigned_int" => return Some("u64".to_owned()),
        "int64_t" | "share_type" => return Some("i64".to_owned()),
        "block_id_type" => return Some("String".to_owned()),
        "chain_id_type" => return Some("String".to_owned()),
        "commitment_type" | "fc::ecc::commitment_type" => return Some("String".to_owned()),
        "public_key_type" => return Some("String".to_owned()),
        "fc::uint128_t" | "uint128_t" => return Some("u128".to_owned()),
        "time_point_sec" => return Some("String".to_owned()),
        "vesting_balance_type" => return Some("String".to_owned()),
        "vesting_policy" => return Some("graphene_protocol::VestingPolicy".to_owned()),
        "worker_type" => {
            return Some(
                "graphene_protocol::WorkerType<crate::types::vesting_balance::Id>".to_owned(),
            );
        }
        "ticket_status" => return Some("String".to_owned()),
        "ticket_type" => return Some("String".to_owned()),
        "transaction" => return Some("graphene_protocol::Transaction".to_owned()),
        "transaction_id_type" => return Some("String".to_owned()),
        "signed_transaction" => return Some("graphene_protocol::SignedTransaction".to_owned()),
        "vote_id_type" => return Some("String".to_owned()),
        "authority" => {
            return Some("graphene_protocol::Authority<crate::types::account::Id>".to_owned());
        }
        "chain_parameters" => return Some("graphene_protocol::ChainParameters".to_owned()),
        "immutable_chain_parameters" => {
            return Some("graphene_protocol::ImmutableChainParameters".to_owned());
        }
        "condition_info" => return Some("graphene_protocol::HtlcConditions".to_owned()),
        "limit_order_auto_action" => {
            return Some(
                "graphene_protocol::LimitOrderAutoAction<crate::types::asset::Id>".to_owned(),
            );
        }
        "linear_vesting_policy" => {
            return Some("graphene_protocol::LinearVestingPolicy".to_owned());
        }
        "memo_data" => return Some("graphene_protocol::MemoData".to_owned()),
        "operation" => return Some("graphene_protocol::Operation".to_owned()),
        "operation_result" => return Some("graphene_protocol::OperationResult".to_owned()),
        "restriction" => return Some("graphene_protocol::Restriction".to_owned()),
        "special_authority" => {
            return Some("graphene_protocol::SpecialAuthority<crate::types::asset::Id>".to_owned());
        }
        "transfer_info" => {
            return Some(
                "graphene_protocol::HtlcTransfer<crate::types::account::Id, crate::types::asset::Id>".to_owned(),
            );
        }
        _ => {}
    }

    if let Some(inner) = template_argument(&normalized, "optional") {
        return map_cpp_type_to_rust(inner).map(|rust_type| format!("Option<{rust_type}>"));
    }

    if let Some(inner) = template_argument(&normalized, "fc::optional") {
        return map_cpp_type_to_rust(inner).map(|rust_type| format!("Option<{rust_type}>"));
    }

    if let Some(inner) = template_argument(&normalized, "pair") {
        if let Some((first_type, second_type)) = split_template_pair(inner) {
            let first_type = map_cpp_type_to_rust(first_type)?;
            let second_type = map_cpp_type_to_rust(second_type)?;
            return Some(format!("({first_type}, {second_type})"));
        }
    }

    if let Some(inner) = template_argument(&normalized, "flat_map") {
        if let Some((key_type, value_type)) = split_template_pair(inner) {
            let key_type = map_cpp_type_to_rust(key_type)?;
            let value_type = map_cpp_type_to_rust(value_type)?;
            return Some(format!("Vec<({key_type}, {value_type})>"));
        }
    }

    for container in ["flat_set", "set", "vector"] {
        if let Some(inner) = template_argument(&normalized, container) {
            return map_cpp_type_to_rust(inner).map(|rust_type| format!("Vec<{rust_type}>"));
        }
    }

    normalized
        .strip_suffix("_id_type")
        .map(|family| format!("crate::types::{family}::Id"))
}

/// Apply C++ type mapping to parsed reflected fields.
pub fn map_fields_to_rust(fields: &[CppField]) -> Result<Vec<RustField>, String> {
    map_fields_to_rust_for_chain("", fields)
}

/// Apply C++ type mapping to parsed reflected fields for a specific chain crate.
///
/// Generic mapping keeps operation/transaction wrappers raw-compatible through
/// `graphene_protocol`. BitShares object generation opts into chain-local typed
/// transaction models where those models exist.
pub fn map_fields_to_rust_for_chain(
    chain_name: &str,
    fields: &[CppField],
) -> Result<Vec<RustField>, String> {
    fields
        .iter()
        .map(|field| {
            let Some(rust_type) = map_cpp_type_to_rust_for_chain(chain_name, &field.cpp_type)
            else {
                return Err(format!(
                    "no Rust type mapping for C++ field {}: {}",
                    field.name, field.cpp_type
                ));
            };
            Ok(RustField {
                name: field.name.clone(),
                rust_type,
            })
        })
        .collect()
}

fn map_cpp_type_to_rust_for_chain(chain_name: &str, cpp_type: &str) -> Option<String> {
    if chain_name == "bitshares" {
        match normalize_cpp_type(cpp_type).as_str() {
            "operation" => return Some("crate::operations::Operation".to_owned()),
            "operation_result" => return Some("crate::transaction::OperationResult".to_owned()),
            "transaction" => return Some("crate::transaction::Transaction".to_owned()),
            "signed_transaction" => {
                return Some("crate::transaction::SignedTransaction".to_owned());
            }
            _ => {}
        }
    }

    map_cpp_type_to_rust(cpp_type)
}

/// Render a generated Rust `Object` struct from already-mapped fields.
///
/// The generated struct always includes `id: Id` first. Other fields preserve FC reflection order.
pub fn render_object_struct(fields: &[RustField]) -> String {
    let mut output = String::new();
    output.push_str("#[derive(Clone, Debug, PartialEq, serde::Deserialize)]\n");
    output.push_str("pub struct Object {\n");
    output.push_str("    pub id: Id,\n");

    for field in fields {
        if field.name == "id" {
            continue;
        }
        if let Some(deserializer) = numeric_deserializer(&field.rust_type) {
            output.push_str(&format!(
                "    #[serde(deserialize_with = \"graphene_protocol::{deserializer}\")]\n"
            ));
        }
        output.push_str(&format!("    pub {}: {},\n", field.name, field.rust_type));
    }

    output.push_str("}\n");
    output
}

fn numeric_deserializer(rust_type: &str) -> Option<&'static str> {
    match rust_type {
        "i64" => Some("i64_from_number_or_string"),
        "u64" => Some("u64_from_number_or_string"),
        "u128" => Some("u128_from_number_or_string"),
        _ => None,
    }
}

/// Parse `FC_REFLECT...` macros from C++ source text.
///
/// This parser uses FC reflection as the source of field order. It intentionally does not infer
/// fields from class declarations; use [`parse_reflected_class_fields`] afterwards to recover C++
/// source types for these reflected names.
pub fn parse_reflected_objects(source: &str) -> Result<Vec<ReflectedObject>, String> {
    let mut reflected_objects = Vec::new();
    let mut remaining = source;

    while let Some(relative_start) = remaining.find("FC_REFLECT") {
        remaining = &remaining[relative_start..];
        let Some(open_paren) = remaining.find('(') else {
            break;
        };
        let macro_name = remaining[..open_paren].trim();
        let after_open = &remaining[open_paren + 1..];
        let Some((body, consumed)) = take_balanced_parentheses_body(after_open) else {
            break;
        };

        if let Some(reflected_object) = parse_reflect_body(macro_name, body)? {
            reflected_objects.push(reflected_object);
        }
        remaining = &after_open[consumed..];
    }

    Ok(reflected_objects)
}

fn parse_reflect_body(macro_name: &str, body: &str) -> Result<Option<ReflectedObject>, String> {
    let Some(first_comma) = find_next_code_comma(body, 0) else {
        return Ok(None);
    };
    let cpp_type = body[..first_comma].trim();
    if !cpp_type.starts_with("graphene::") {
        return Ok(None);
    }

    let fields_source = if macro_name.contains("DERIVED") {
        let Some(second_comma) = find_next_code_comma(body, first_comma + 1) else {
            return Err(format!(
                "{macro_name} for {cpp_type} is missing reflected field list"
            ));
        };
        &body[second_comma + 1..]
    } else {
        &body[first_comma + 1..]
    };

    Ok(Some(ReflectedObject {
        cpp_type: normalize_cpp_type(cpp_type),
        fields: parse_reflected_field_names(fields_source),
    }))
}

fn parse_reflected_field_names(source: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut index = 0usize;
    let bytes = source.as_bytes();

    while index < bytes.len() {
        if bytes[index] == b'(' {
            if let Some(close_offset) = source[index + 1..].find(')') {
                let name = source[index + 1..index + 1 + close_offset].trim();
                if is_family_name(name) {
                    fields.push(name.to_owned());
                }
                index += close_offset + 2;
                continue;
            }
            break;
        }
        index += 1;
    }

    fields
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BodySpan<'a> {
    body: &'a str,
    start_line: usize,
}

fn class_or_struct_body<'a>(source: &'a str, type_name: &str) -> Option<BodySpan<'a>> {
    for keyword in ["class", "struct"] {
        let marker = format!("{keyword} {type_name}");
        let mut search_from = 0usize;

        while let Some(relative_start) = source[search_from..].find(&marker) {
            let type_start = search_from + relative_start;
            if !is_identifier_boundary(source, type_start, marker.len()) {
                search_from = type_start + marker.len();
                continue;
            }

            let after_type = &source[type_start + marker.len()..];
            let next_semicolon = after_type.find(';');
            let Some(open_brace) = after_type.find('{') else {
                return None;
            };

            if next_semicolon.is_some_and(|semicolon| semicolon < open_brace) {
                search_from = type_start + marker.len();
                continue;
            }

            let open_brace_index = type_start + marker.len() + open_brace;
            let after_open = &after_type[open_brace + 1..];
            return take_balanced_brace_body(after_open).map(|(body, _consumed)| BodySpan {
                body,
                start_line: line_number_at(source, open_brace_index) + 1,
            });
        }
    }

    None
}

fn take_balanced_brace_body(source_after_open: &str) -> Option<(&str, usize)> {
    let mut depth = 1usize;

    for (index, character) in source_after_open.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&source_after_open[..index], index + 1));
                }
            }
            _ => {}
        }
    }

    None
}

fn parse_top_level_declarations(
    class_body: &str,
    source_file: &str,
    body_start_line: usize,
) -> Vec<DeclaredCppField> {
    let class_body = strip_cpp_comments_preserving_layout(class_body);
    let mut declarations = Vec::new();
    let mut statement_start = 0usize;
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut skipping_inline_function_body = false;

    for (index, character) in class_body.char_indices() {
        match character {
            '<' if brace_depth == 0 && paren_depth == 0 => angle_depth += 1,
            '>' if angle_depth > 0 && brace_depth == 0 && paren_depth == 0 => angle_depth -= 1,
            '(' if brace_depth == 0 => paren_depth += 1,
            ')' if paren_depth > 0 && brace_depth == 0 => paren_depth -= 1,
            '{' => {
                if brace_depth == 0 && class_body[statement_start..index].contains('(') {
                    skipping_inline_function_body = true;
                }
                brace_depth += 1;
            }
            '}' if brace_depth > 0 => {
                brace_depth -= 1;
                if brace_depth == 0 && skipping_inline_function_body {
                    statement_start = index + 1;
                    skipping_inline_function_body = false;
                }
            }
            ';' if angle_depth == 0 && paren_depth == 0 && brace_depth == 0 => {
                let statement = class_body[statement_start..index].trim();
                let source_line =
                    body_start_line + class_body[..statement_start].matches('\n').count();
                declarations.extend(parse_field_declarations(
                    statement,
                    source_file,
                    source_line,
                ));
                statement_start = index + 1;
            }
            _ => {}
        }
    }

    declarations
}

fn parse_field_declarations(
    statement: &str,
    source_file: &str,
    source_line: usize,
) -> Vec<DeclaredCppField> {
    let declaration_without_initializer = trim_initializer(statement).trim();
    let declaration = declaration_without_initializer
        .rsplit(':')
        .next()
        .unwrap_or(declaration_without_initializer)
        .trim();
    if declaration.is_empty()
        || declaration.starts_with("static ")
        || declaration.starts_with("typedef ")
        || declaration.starts_with("using ")
        || declaration.starts_with("struct ")
        || declaration.starts_with("class ")
        || declaration.contains('(')
    {
        return Vec::new();
    }

    let declarators = split_declarators(declaration);
    if declarators.is_empty() {
        return Vec::new();
    }

    let Some((first_type, first_name)) = split_type_and_name(declarators[0]) else {
        return Vec::new();
    };
    if first_type.is_empty() || !is_family_name(first_name) {
        return Vec::new();
    }

    let cpp_type = normalize_cpp_type(first_type);
    let mut fields = vec![DeclaredCppField {
        name: first_name.to_owned(),
        cpp_type: cpp_type.clone(),
        source_file: source_file.to_owned(),
        source_line,
    }];

    for declarator in declarators.iter().skip(1) {
        let declarator = trim_initializer(declarator).trim();
        let name = declarator.trim_start_matches(['*', '&']).trim();
        if is_family_name(name) {
            fields.push(DeclaredCppField {
                name: name.to_owned(),
                cpp_type: cpp_type.clone(),
                source_file: source_file.to_owned(),
                source_line,
            });
        }
    }

    fields
}

fn split_type_and_name(declarator: &str) -> Option<(&str, &str)> {
    let mut parts = declarator.rsplitn(2, char::is_whitespace);
    let name = parts.next()?.trim().trim_start_matches(['*', '&']).trim();
    let cpp_type = parts.next()?.trim();
    Some((cpp_type, name))
}

fn trim_initializer(statement: &str) -> &str {
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    for (index, character) in statement.char_indices() {
        match character {
            '<' if paren_depth == 0 && bracket_depth == 0 => angle_depth += 1,
            '>' if angle_depth > 0 && paren_depth == 0 && bracket_depth == 0 => angle_depth -= 1,
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' if bracket_depth > 0 => bracket_depth -= 1,
            '=' | '{' if angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 => {
                return &statement[..index];
            }
            _ => {}
        }
    }

    statement
}

fn split_declarators(declaration: &str) -> Vec<&str> {
    let mut declarators = Vec::new();
    let mut start = 0usize;
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;

    for (index, character) in declaration.char_indices() {
        match character {
            '<' if paren_depth == 0 && brace_depth == 0 => angle_depth += 1,
            '>' if angle_depth > 0 && paren_depth == 0 && brace_depth == 0 => angle_depth -= 1,
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            '{' => brace_depth += 1,
            '}' if brace_depth > 0 => brace_depth -= 1,
            ',' if angle_depth == 0 && paren_depth == 0 && brace_depth == 0 => {
                declarators.push(declaration[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    let tail = declaration[start..].trim();
    if !tail.is_empty() {
        declarators.push(tail);
    }

    declarators
}

fn strip_cpp_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut index = 0usize;

    while index < source.len() {
        if source[index..].starts_with("/*") {
            if let Some(comment_end) = source[index + 2..].find("*/") {
                index += comment_end + 4;
                output.push(' ');
                continue;
            }
        }

        if source[index..].starts_with("//") {
            if let Some(line_end) = source[index + 2..].find('\n') {
                index += line_end + 2;
                output.push('\n');
                continue;
            }
            break;
        }

        let character = source[index..].chars().next().expect("valid char boundary");
        output.push(character);
        index += character.len_utf8();
    }

    output
}

fn strip_cpp_comments_preserving_layout(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut index = 0usize;

    while index < source.len() {
        if source[index..].starts_with("/*") {
            if let Some(comment_end) = source[index + 2..].find("*/") {
                let comment = &source[index..index + comment_end + 4];
                for character in comment.chars() {
                    output.push(if character == '\n' { '\n' } else { ' ' });
                }
                index += comment_end + 4;
                continue;
            }
        }

        if source[index..].starts_with("//") {
            if let Some(line_end) = source[index + 2..].find('\n') {
                let comment = &source[index..index + line_end + 2];
                output.extend(comment.chars().map(
                    |character| {
                        if character == '\n' { '\n' } else { ' ' }
                    },
                ));
                index += line_end + 2;
                continue;
            }
            output.extend(source[index..].chars().map(
                |character| {
                    if character == '\n' { '\n' } else { ' ' }
                },
            ));
            break;
        }

        let character = source[index..].chars().next().expect("valid char boundary");
        output.push(character);
        index += character.len_utf8();
    }

    output
}

fn line_number_at(source: &str, index: usize) -> usize {
    source[..index].matches('\n').count() + 1
}

fn unqualified_cpp_name(cpp_type: &str) -> &str {
    cpp_type.rsplit("::").next().unwrap_or(cpp_type).trim()
}

fn normalize_cpp_type(cpp_type: &str) -> String {
    cpp_type
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("< ", "<")
        .replace(" >", ">")
}

fn template_argument<'a>(cpp_type: &'a str, template: &str) -> Option<&'a str> {
    let prefix = format!("{template}<");
    cpp_type
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix('>'))
        .map(str::trim)
}

fn split_template_pair(arguments: &str) -> Option<(&str, &str)> {
    let comma = find_next_code_comma(arguments, 0)?;
    Some((arguments[..comma].trim(), arguments[comma + 1..].trim()))
}

fn parse_define_ids_body(body: &str) -> Result<Vec<ObjectFamily>, String> {
    let Some((object_space_name, family_names_source)) = define_ids_space_and_names(body) else {
        return Ok(Vec::new());
    };
    let Some(object_space) = object_space_from_name(object_space_name) else {
        return Ok(Vec::new());
    };

    parse_family_names(family_names_source, object_space)
}

fn define_ids_space_and_names(body: &str) -> Option<(&str, &str)> {
    let first_comma = find_next_code_comma(body, 0)?;
    let second_comma = find_next_code_comma(body, first_comma + 1)?;
    let third_comma = find_next_code_comma(body, second_comma + 1)?;

    Some((
        body[first_comma + 1..second_comma].trim(),
        &body[third_comma + 1..],
    ))
}

fn object_space_from_name(object_space_name: &str) -> Option<u8> {
    match object_space_name {
        "relative_protocol_ids" => Some(0),
        "protocol_ids" => Some(1),
        "implementation_ids" => Some(2),
        _ => None,
    }
}

fn find_next_code_comma(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = start;

    while index < bytes.len() {
        if source[index..].starts_with("/*") {
            let comment_end = source[index + 2..].find("*/")?;
            index += comment_end + 4;
            continue;
        }

        if source[index..].starts_with("//") {
            let line_end = source[index + 2..]
                .find('\n')
                .map(|offset| index + 2 + offset)
                .unwrap_or(source.len());
            index = line_end;
            continue;
        }

        if bytes[index] == b',' {
            return Some(index);
        }

        index += 1;
    }

    None
}

fn parse_family_names(source: &str, object_space: u8) -> Result<Vec<ObjectFamily>, String> {
    let mut families = Vec::new();
    let mut next_type_id = 0u8;
    let mut pending_comment_type_id = None;
    let mut index = 0;
    let bytes = source.as_bytes();

    while index < bytes.len() {
        if source[index..].starts_with("/*") {
            if let Some(comment_end) = source[index + 2..].find("*/") {
                let comment = &source[index + 2..index + 2 + comment_end];
                pending_comment_type_id = parse_type_id_from_comment(comment);
                index += comment_end + 4;
                continue;
            }
            break;
        }

        if bytes[index] == b'(' {
            if let Some(close_offset) = source[index + 1..].find(')') {
                let name = source[index + 1..index + 1 + close_offset].trim();
                if is_family_name(name) {
                    let type_id = next_type_id;
                    if let Some(comment_type_id) = pending_comment_type_id {
                        if comment_type_id != type_id {
                            return Err(format!(
                                "GRAPHENE_DEFINE_IDS comment type id for {name} ({comment_type_id}) disagrees with declaration order ({type_id})"
                            ));
                        }
                    }
                    families.push(ObjectFamily {
                        object_space,
                        type_id,
                        name: name.to_owned(),
                    });
                    next_type_id = next_type_id.saturating_add(1);
                    pending_comment_type_id = None;
                }
                index += close_offset + 2;
                continue;
            }
            break;
        }

        index += 1;
    }

    Ok(families)
}

fn static_variant_alias_body<'a>(source: &'a str, alias: &str) -> Result<&'a str, String> {
    let mut search_offset = 0usize;

    while search_offset < source.len() {
        let using_offset = source[search_offset..].find("using");
        let typedef_offset = source[search_offset..].find("typedef");
        let next = match (using_offset, typedef_offset) {
            (Some(using_offset), Some(typedef_offset)) if using_offset <= typedef_offset => {
                Some((search_offset + using_offset, "using"))
            }
            (Some(_), Some(typedef_offset)) => Some((search_offset + typedef_offset, "typedef")),
            (Some(using_offset), None) => Some((search_offset + using_offset, "using")),
            (None, Some(typedef_offset)) => Some((search_offset + typedef_offset, "typedef")),
            (None, None) => None,
        };

        let Some((keyword_index, keyword)) = next else {
            break;
        };
        if !is_identifier_boundary(source, keyword_index, keyword.len()) {
            search_offset = keyword_index + keyword.len();
            continue;
        }

        let remainder = &source[keyword_index + keyword.len()..];
        if keyword == "using" {
            match using_static_variant_alias_body(remainder, alias)? {
                Some(body) => return Ok(body),
                None => search_offset = keyword_index + keyword.len(),
            }
        } else {
            match typedef_static_variant_alias_body(remainder, alias)? {
                Some(body) => return Ok(body),
                None => search_offset = keyword_index + keyword.len(),
            }
        }
    }

    Err(format!(
        "could not find static_variant alias {alias} as using or typedef"
    ))
}

fn using_static_variant_alias_body<'a>(
    remainder: &'a str,
    alias: &str,
) -> Result<Option<&'a str>, String> {
    let mut remainder = remainder.trim_start();
    if !remainder.starts_with(alias) || !is_prefix_identifier_boundary(remainder, alias.len()) {
        return Ok(None);
    }

    remainder = remainder[alias.len()..].trim_start();
    if !remainder.starts_with('=') {
        return Ok(None);
    }

    remainder = remainder[1..].trim_start();
    let Some(after_static_variant) = strip_static_variant_prefix(remainder) else {
        return Err(format!(
            "using alias {alias} is not assigned from fc::static_variant"
        ));
    };

    static_variant_body_after_prefix(after_static_variant, alias).map(Some)
}

fn typedef_static_variant_alias_body<'a>(
    remainder: &'a str,
    alias: &str,
) -> Result<Option<&'a str>, String> {
    let remainder = remainder.trim_start();
    let Some(after_static_variant) = strip_static_variant_prefix(remainder) else {
        return Ok(None);
    };
    let body = static_variant_body_after_prefix(after_static_variant, alias)?;
    let tail = after_static_variant[1 + body.len() + 1..].trim_start();

    if !tail.starts_with(alias) || !is_prefix_identifier_boundary(tail, alias.len()) {
        return Ok(None);
    }

    Ok(Some(body))
}

fn strip_static_variant_prefix(source: &str) -> Option<&str> {
    for prefix in ["fc::static_variant", "static_variant"] {
        if let Some(rest) = source.strip_prefix(prefix) {
            return Some(rest.trim_start());
        }
    }

    None
}

fn static_variant_body_after_prefix<'a>(
    remainder: &'a str,
    alias: &str,
) -> Result<&'a str, String> {
    if !remainder.starts_with('<') {
        return Err(format!(
            "static_variant alias {alias} is missing opening '<'"
        ));
    }

    take_balanced_angle_body(&remainder[1..])
        .ok_or_else(|| format!("static_variant alias {alias} has an unterminated variant list"))
}

fn is_identifier_boundary(source: &str, start: usize, len: usize) -> bool {
    let before = source[..start].chars().next_back();
    let after = source[start + len..].chars().next();
    before.is_none_or(|character| !is_cpp_identifier_character(character))
        && after.is_none_or(|character| !is_cpp_identifier_character(character))
}

fn is_prefix_identifier_boundary(source: &str, len: usize) -> bool {
    source[len..]
        .chars()
        .next()
        .is_none_or(|character| !is_cpp_identifier_character(character))
}

fn is_cpp_identifier_character(character: char) -> bool {
    character == '_' || character.is_ascii_alphanumeric()
}

fn take_balanced_angle_body(source_after_open: &str) -> Option<&str> {
    let mut angle_depth = 1usize;
    let mut index = 0usize;

    while index < source_after_open.len() {
        let remaining = &source_after_open[index..];
        if remaining.starts_with("//") {
            if let Some(newline_offset) = remaining.find('\n') {
                index += newline_offset + 1;
            } else {
                return None;
            }
            continue;
        }
        if remaining.starts_with("/*") {
            let comment_end = remaining.find("*/")?;
            index += comment_end + 2;
            continue;
        }

        let character = remaining.chars().next()?;
        match character {
            '<' => angle_depth += 1,
            '>' => {
                angle_depth -= 1;
                if angle_depth == 0 {
                    return Some(&source_after_open[..index]);
                }
            }
            _ => {}
        }
        index += character.len_utf8();
    }

    None
}

fn split_static_variant_items(body: &str) -> Result<Vec<&str>, String> {
    let mut items = Vec::new();
    let mut item_start = 0usize;
    let mut index = 0usize;
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;

    while index < body.len() {
        let remaining = &body[index..];
        if remaining.starts_with("//") {
            if let Some(newline_offset) = remaining.find('\n') {
                index += newline_offset + 1;
            } else {
                index = body.len();
            }
            continue;
        }
        if remaining.starts_with("/*") {
            let Some(comment_end) = remaining.find("*/") else {
                return Err("unterminated block comment in variant list".to_owned());
            };
            index += comment_end + 2;
            continue;
        }

        let character = remaining
            .chars()
            .next()
            .expect("index should point at a valid character boundary");
        match character {
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ',' if angle_depth == 0
                && paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0 =>
            {
                let mut split_after = index;
                let after_comma = &body[index + 1..];
                let horizontal_ws_len = after_comma
                    .char_indices()
                    .take_while(|(_, character)| *character == ' ' || *character == '\t')
                    .map(|(_, character)| character.len_utf8())
                    .sum::<usize>();
                let after_horizontal_ws = &after_comma[horizontal_ws_len..];
                if after_horizontal_ws.starts_with("//") {
                    if let Some(newline_offset) = after_horizontal_ws.find('\n') {
                        split_after = index + 1 + horizontal_ws_len + newline_offset;
                    } else {
                        split_after = body.len();
                    }
                }

                items.push(body[item_start..split_after].trim());
                item_start = if split_after == body.len() {
                    body.len()
                } else {
                    split_after + 1
                };
                index = item_start;
                continue;
            }
            _ => {}
        }
        index += character.len_utf8();
    }

    if item_start <= body.len() {
        let item = body[item_start..].trim();
        if !item.is_empty() {
            items.push(item);
        }
    }

    Ok(items)
}

fn parse_variant_tag_comment(item: &str) -> Option<u16> {
    let mut search_offset = 0usize;
    while let Some(comment_start_offset) = item[search_offset..].find("/*") {
        let comment_start = search_offset + comment_start_offset;
        let comment_body_start = comment_start + 2;
        let Some(comment_end_offset) = item[comment_body_start..].find("*/") else {
            break;
        };
        let comment_end = comment_body_start + comment_end_offset;
        if let Ok(tag) = item[comment_body_start..comment_end].trim().parse() {
            return Some(tag);
        }
        search_offset = comment_end + 2;
    }
    None
}

fn parse_type_id_from_comment(comment: &str) -> Option<u8> {
    let mut parts = comment.trim().split('.');
    let _space = parts.next()?;
    parts.next()?.trim().parse().ok()
}

fn is_family_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn take_balanced_parentheses_body(source_after_open: &str) -> Option<(&str, usize)> {
    let mut depth = 1usize;

    for (index, character) in source_after_open.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&source_after_open[..index], index + 1));
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{
        CppField, ObjectFamily, OperationDeclaration, OperationDeclarationError, OperationField,
        OperationFieldClassification, OperationModelReportRow, OperationVariant,
        ProtocolHeaderSource, ReflectedObject, RustField, map_cpp_type_to_rust, map_fields_to_rust,
        map_fields_to_rust_for_chain, map_operation_field, map_operation_field_for_operation,
        parse_object_families, parse_operation_declarations, parse_operation_variants,
        parse_reflected_class_fields, parse_reflected_objects, parse_static_variant_alias,
        render_object_id_module, render_object_struct, render_operation_model_skips_report,
        render_operation_structs_module, render_operation_variants_module, render_types_mod,
    };

    #[test]
    fn parses_operation_variant_tags_from_declaration_order() {
        let source = r#"
            using operation = fc::static_variant<
                /*  0 */ transfer_operation,
                /*  1 */ limit_order_create_operation,
                /*  2 */ fill_order_operation // VIRTUAL
            >;
        "#;

        assert_eq!(
            parse_operation_variants(source).expect("operation variants should parse"),
            vec![
                OperationVariant {
                    tag: 0,
                    cpp_type: "transfer_operation".to_owned(),
                    is_virtual: false,
                },
                OperationVariant {
                    tag: 1,
                    cpp_type: "limit_order_create_operation".to_owned(),
                    is_virtual: false,
                },
                OperationVariant {
                    tag: 2,
                    cpp_type: "fill_order_operation".to_owned(),
                    is_virtual: true,
                },
            ]
        );
    }

    #[test]
    fn operation_variant_comments_are_optional_but_validated_when_present() {
        let source = r#"
            using operation_result = fc::static_variant<
                void_result,
                /* 1 */ object_id_type
            >;
        "#;

        assert_eq!(
            parse_static_variant_alias(source, "operation_result")
                .expect("missing comment should not affect declaration order"),
            vec![
                OperationVariant {
                    tag: 0,
                    cpp_type: "void_result".to_owned(),
                    is_virtual: false,
                },
                OperationVariant {
                    tag: 1,
                    cpp_type: "object_id_type".to_owned(),
                    is_virtual: false,
                },
            ]
        );

        let error = parse_static_variant_alias(
            r#"
                using operation = fc::static_variant<
                    /* 1 */ transfer_operation
                >;
            "#,
            "operation",
        )
        .expect_err("comment/order mismatch should fail");
        assert!(error.contains("operation"));
        assert!(error.contains("transfer_operation"));
        assert!(error.contains("comment tag 1 disagrees with declaration order 0"));
    }

    #[test]
    fn operation_variant_parser_ignores_nested_commas_and_comments() {
        let source = r#"
            using operation = fc::static_variant<
                /* 0 */ namespaced_operation<alpha, beta>,
                /* 1 */ invoked_operation(foo, bar),
                /* 2 */ commented_operation /* comment, with comma */,
                /* 3 */ line_commented_operation // comment, with comma
            >;
        "#;

        assert_eq!(
            parse_static_variant_alias(source, "operation").expect("variants should parse"),
            vec![
                OperationVariant {
                    tag: 0,
                    cpp_type: "namespaced_operation<alpha, beta>".to_owned(),
                    is_virtual: false,
                },
                OperationVariant {
                    tag: 1,
                    cpp_type: "invoked_operation(foo, bar)".to_owned(),
                    is_virtual: false,
                },
                OperationVariant {
                    tag: 2,
                    cpp_type: "commented_operation".to_owned(),
                    is_virtual: false,
                },
                OperationVariant {
                    tag: 3,
                    cpp_type: "line_commented_operation".to_owned(),
                    is_virtual: false,
                },
            ]
        );
    }

    #[test]
    fn operation_variant_parser_rejects_malformed_inputs_without_panicking() {
        assert!(parse_static_variant_alias("", "operation").is_err());
        assert!(
            parse_static_variant_alias(
                "using operation = fc::static_variant<foo, bar;",
                "operation"
            )
            .is_err()
        );
        assert!(
            parse_static_variant_alias("using operation = fc::static_variant<>;", "operation")
                .is_err()
        );
    }

    #[test]
    fn static_variant_parser_handles_bitshares_typedef_alias_forms_and_ordering() {
        let vesting_source = include_str!(
            "../../../../chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/vesting.hpp"
        );
        let worker_source = include_str!(
            "../../../../chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/worker.hpp"
        );
        let assert_source = include_str!(
            "../../../../chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/assert.hpp"
        );

        assert_eq!(
            parse_static_variant_alias(vesting_source, "vesting_policy_initializer")
                .expect("fc::static_variant typedef should parse")
                .iter()
                .map(|variant| (variant.tag, variant.cpp_type.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (0, "linear_vesting_policy_initializer"),
                (1, "cdd_vesting_policy_initializer"),
                (2, "instant_vesting_policy_initializer"),
            ]
        );
        assert_eq!(
            parse_static_variant_alias(worker_source, "worker_initializer")
                .expect("unqualified static_variant typedef should parse")
                .iter()
                .map(|variant| (variant.tag, variant.cpp_type.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (0, "refund_worker_initializer"),
                (1, "vesting_balance_worker_initializer"),
                (2, "burn_worker_initializer"),
            ]
        );
        assert_eq!(
            parse_static_variant_alias(assert_source, "predicate")
                .expect("predicate typedef should parse")
                .iter()
                .map(|variant| (variant.tag, variant.cpp_type.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (0, "account_name_eq_lit_predicate"),
                (1, "asset_symbol_eq_lit_predicate"),
                (2, "block_id_predicate"),
            ]
        );
    }

    #[test]
    fn static_variant_parser_does_not_treat_unsupported_typedefs_as_alias_mappings() {
        let source = r#"
            typedef vector<predicate> predicate_list;
            typedef flat_set<future_extensions> extensions_type;
        "#;

        let error = parse_static_variant_alias(source, "predicate_list")
            .expect_err("non-static-variant typedefs should not produce variant mappings");
        assert!(error.contains("could not find static_variant alias predicate_list"));
    }

    #[test]
    fn operation_variant_parser_handles_bitshares_operations_header() {
        let source = include_str!(
            "../../../../chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/operations.hpp"
        );
        let variants = parse_operation_variants(source).expect("operations.hpp should parse");

        assert_eq!(variants.first().expect("first variant").tag, 0);
        assert_eq!(
            variants.first().expect("first variant").cpp_type,
            "transfer_operation"
        );
        assert_eq!(variants.len(), 78);
        assert_eq!(variants.last().expect("last variant").tag, 77);
        assert_eq!(
            variants.last().expect("last variant").cpp_type,
            "limit_order_update_operation"
        );
        assert_eq!(
            variants
                .iter()
                .filter(|variant| variant.is_virtual)
                .map(|variant| variant.cpp_type.as_str())
                .collect::<Vec<_>>(),
            vec![
                "fill_order_operation",
                "asset_settle_cancel_operation",
                "fba_distribute_operation",
                "execute_bid_operation",
                "htlc_redeemed_operation",
                "htlc_refund_operation",
                "credit_deal_expired_operation",
            ]
        );
        for (expected_tag, variant) in variants.iter().enumerate() {
            assert_eq!(variant.tag as usize, expected_tag);
        }
    }

    #[test]
    fn renders_operation_variant_module_with_stable_data_only_table() {
        let source = render_operation_variants_module(
            "bitshares",
            &[
                OperationVariant {
                    tag: 0,
                    cpp_type: "transfer_operation".to_owned(),
                    is_virtual: false,
                },
                OperationVariant {
                    tag: 2,
                    cpp_type: "fill_order_operation".to_owned(),
                    is_virtual: true,
                },
            ],
        );

        assert_eq!(
            source,
            concat!(
                "// @generated by graphene-codegen; do not edit by hand.\n",
                "// Source chain: bitshares operation static_variant tags\n",
                "\n",
                "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n",
                "pub struct OperationVariantSpec {\n",
                "    pub tag: u16,\n",
                "    pub cpp_type: &'static str,\n",
                "    pub is_virtual: bool,\n",
                "}\n",
                "\n",
                "pub const BITSHARES_OPERATION_VARIANTS: &[OperationVariantSpec] = &[\n",
                "    OperationVariantSpec { tag: 0, cpp_type: \"transfer_operation\", is_virtual: false },\n",
                "    OperationVariantSpec { tag: 2, cpp_type: \"fill_order_operation\", is_virtual: true },\n",
                "];\n",
            )
        );
    }

    #[test]
    fn renders_empty_operation_variant_module_as_valid_empty_table() {
        assert_eq!(
            render_operation_variants_module("bitshares", &[]),
            concat!(
                "// @generated by graphene-codegen; do not edit by hand.\n",
                "// Source chain: bitshares operation static_variant tags\n",
                "\n",
                "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n",
                "pub struct OperationVariantSpec {\n",
                "    pub tag: u16,\n",
                "    pub cpp_type: &'static str,\n",
                "    pub is_virtual: bool,\n",
                "}\n",
                "\n",
                "pub const BITSHARES_OPERATION_VARIANTS: &[OperationVariantSpec] = &[\n",
                "];\n",
            )
        );
    }

    #[test]
    fn rendered_bitshares_operation_variant_module_contains_boundary_tags() {
        let source = include_str!(
            "../../../../chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/operations.hpp"
        );
        let variants = parse_operation_variants(source).expect("operations.hpp should parse");
        let rendered = render_operation_variants_module("bitshares", &variants);

        assert!(rendered.contains(
            "OperationVariantSpec { tag: 0, cpp_type: \"transfer_operation\", is_virtual: false }"
        ));
        assert!(rendered.contains(
            "OperationVariantSpec { tag: 37, cpp_type: \"balance_claim_operation\", is_virtual: false }"
        ));
        assert!(rendered.contains(
            "OperationVariantSpec { tag: 77, cpp_type: \"limit_order_update_operation\", is_virtual: false }"
        ));
    }

    #[test]
    fn operation_declaration_parser_uses_struct_reflection_order_and_source_lines() {
        let variants = vec![OperationVariant {
            tag: 0,
            cpp_type: "transfer_operation".to_owned(),
            is_virtual: false,
        }];
        let header = r#"
            namespace graphene { namespace chain {
            struct transfer_operation : public base_operation
            {
               public:
                  asset amount;
                  account_id_type from, to, redeemer;
                  using extensions_type = flat_set<future_extensions>;
                  extensions_type extensions;
            };
            } }

            FC_REFLECT( graphene::chain::transfer_operation, (from)(to)(amount)(extensions) )
        "#;

        assert_eq!(
            parse_operation_declarations(
                &variants,
                &[ProtocolHeaderSource {
                    path: "operations.hpp",
                    source: header,
                }]
            )
            .expect("operation declarations should parse"),
            vec![OperationDeclaration {
                tag: 0,
                cpp_type: "transfer_operation".to_owned(),
                name: "transfer_operation".to_owned(),
                fields: vec![
                    OperationField {
                        name: "from".to_owned(),
                        cpp_type: "account_id_type".to_owned(),
                        source_file: "operations.hpp".to_owned(),
                        source_line: 7,
                    },
                    OperationField {
                        name: "to".to_owned(),
                        cpp_type: "account_id_type".to_owned(),
                        source_file: "operations.hpp".to_owned(),
                        source_line: 7,
                    },
                    OperationField {
                        name: "amount".to_owned(),
                        cpp_type: "asset".to_owned(),
                        source_file: "operations.hpp".to_owned(),
                        source_line: 5,
                    },
                    OperationField {
                        name: "extensions".to_owned(),
                        cpp_type: "extensions_type".to_owned(),
                        source_file: "operations.hpp".to_owned(),
                        source_line: 9,
                    },
                ],
            }]
        );
    }

    #[test]
    fn operation_declaration_parser_skips_nested_types_and_constructor_bodies() {
        let variants = vec![OperationVariant {
            tag: 3,
            cpp_type: "limit_order_create_operation".to_owned(),
            is_virtual: false,
        }];
        let header = r#"
            struct limit_order_create_operation : public base_operation
            {
               struct fee_parameters_type { uint64_t fee = 0; };
               limit_order_create_operation()
                  : fill_or_kill(false)
               {
                  validate();
               }

               asset amount_to_sell;
               asset min_to_receive;
               bool fill_or_kill = false;
            };

            FC_REFLECT( graphene::chain::limit_order_create_operation,
                        (min_to_receive)(amount_to_sell)(fill_or_kill) )
        "#;

        let declarations = parse_operation_declarations(
            &variants,
            &[ProtocolHeaderSource {
                path: "limit_order.hpp",
                source: header,
            }],
        )
        .expect("nested types and constructor bodies should be skipped");

        assert_eq!(
            declarations[0]
                .fields
                .iter()
                .map(|field| (field.name.as_str(), field.cpp_type.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("min_to_receive", "asset"),
                ("amount_to_sell", "asset"),
                ("fill_or_kill", "bool"),
            ]
        );
    }

    #[test]
    fn operation_declaration_parser_reports_missing_reflection_declaration_and_field() {
        let variants = vec![
            OperationVariant {
                tag: 0,
                cpp_type: "transfer_operation".to_owned(),
                is_virtual: false,
            },
            OperationVariant {
                tag: 1,
                cpp_type: "missing_reflection_operation".to_owned(),
                is_virtual: false,
            },
            OperationVariant {
                tag: 2,
                cpp_type: "missing_declaration_operation".to_owned(),
                is_virtual: false,
            },
        ];
        let header = r#"
            struct transfer_operation : public base_operation
            {
               account_id_type from;
            };
            struct missing_reflection_operation : public base_operation
            {
               account_id_type owner;
            };
            FC_REFLECT( graphene::chain::transfer_operation, (from)(to) )
            FC_REFLECT( graphene::chain::missing_declaration_operation, () )
        "#;

        let errors = parse_operation_declarations(
            &variants,
            &[ProtocolHeaderSource {
                path: "operations.hpp",
                source: header,
            }],
        )
        .expect_err("missing operation facts should be reportable");

        assert!(errors.contains(&OperationDeclarationError {
            tag: 0,
            cpp_type: "transfer_operation".to_owned(),
            name: "transfer_operation".to_owned(),
            field_name: Some("to".to_owned()),
            cpp_type_hint: None,
            source_file: Some("operations.hpp".to_owned()),
            source_line: Some(4),
            reason: "reflected field is missing from operation declaration".to_owned(),
        }));
        assert!(errors.contains(&OperationDeclarationError {
            tag: 1,
            cpp_type: "missing_reflection_operation".to_owned(),
            name: "missing_reflection_operation".to_owned(),
            field_name: None,
            cpp_type_hint: None,
            source_file: None,
            source_line: None,
            reason: "missing FC_REFLECT entry for operation".to_owned(),
        }));
        assert!(errors.contains(&OperationDeclarationError {
            tag: 2,
            cpp_type: "missing_declaration_operation".to_owned(),
            name: "missing_declaration_operation".to_owned(),
            field_name: None,
            cpp_type_hint: None,
            source_file: None,
            source_line: None,
            reason: "missing class or struct declaration for reflected operation".to_owned(),
        }));
    }

    #[test]
    fn maps_operation_types_to_protocol_safe_rust_types() {
        assert_eq!(
            map_operation_field(&OperationField {
                name: "seller".to_owned(),
                cpp_type: "account_id_type".to_owned(),
                source_file: "market.hpp".to_owned(),
                source_line: 42,
            }),
            OperationFieldClassification::Typed {
                rust_type: "graphene_protocol::ObjectId".to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "amount_to_sell".to_owned(),
                cpp_type: "asset".to_owned(),
                source_file: "market.hpp".to_owned(),
                source_line: 43,
            }),
            OperationFieldClassification::Typed {
                rust_type: "graphene_protocol::Asset<graphene_protocol::ObjectId>".to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "price".to_owned(),
                cpp_type: "optional<flat_map<account_id_type, pair<uint16_t, asset>>>".to_owned(),
                source_file: "market.hpp".to_owned(),
                source_line: 44,
            }),
            OperationFieldClassification::Typed {
                rust_type: "Option<Vec<(graphene_protocol::ObjectId, (u16, graphene_protocol::Asset<graphene_protocol::ObjectId>))>>".to_owned(),
            }
        );

        let typed_id_mapping = map_operation_field(&OperationField {
            name: "asset_id".to_owned(),
            cpp_type: "asset_id_type".to_owned(),
            source_file: "asset.hpp".to_owned(),
            source_line: 5,
        });
        assert!(!format!("{typed_id_mapping:?}").contains("crate::types"));
    }

    #[test]
    fn maps_s04_operation_helpers_to_protocol_safe_types() {
        for (field_name, cpp_type, rust_type) in [
            (
                "new_parameters",
                "chain_parameters",
                "graphene_protocol::ChainParameters",
            ),
            (
                "policy",
                "vesting_policy_initializer",
                "graphene_protocol::VestingPolicyInitializer",
            ),
            (
                "initializer",
                "worker_initializer",
                "graphene_protocol::WorkerInitializer",
            ),
            (
                "predicates",
                "vector<predicate>",
                "Vec<graphene_protocol::Predicate<graphene_protocol::ObjectId, graphene_protocol::ObjectId>>",
            ),
        ] {
            assert_eq!(
                map_operation_field(&OperationField {
                    name: field_name.to_owned(),
                    cpp_type: cpp_type.to_owned(),
                    source_file: "s04.hpp".to_owned(),
                    source_line: 1,
                }),
                OperationFieldClassification::Typed {
                    rust_type: rust_type.to_owned(),
                }
            );
        }
    }

    #[test]
    fn s04_operation_helpers_emit_generated_operation_field_snippets() {
        let rendered = render_operation_structs_module(&[
            OperationDeclaration {
                tag: 31,
                cpp_type: "committee_member_update_global_parameters_operation".to_owned(),
                name: "committee_member_update_global_parameters_operation".to_owned(),
                fields: vec![OperationField {
                    name: "new_parameters".to_owned(),
                    cpp_type: "chain_parameters".to_owned(),
                    source_file: "committee_member.hpp".to_owned(),
                    source_line: 89,
                }],
            },
            OperationDeclaration {
                tag: 32,
                cpp_type: "vesting_balance_create_operation".to_owned(),
                name: "vesting_balance_create_operation".to_owned(),
                fields: vec![OperationField {
                    name: "policy".to_owned(),
                    cpp_type: "vesting_policy_initializer".to_owned(),
                    source_file: "vesting.hpp".to_owned(),
                    source_line: 82,
                }],
            },
            OperationDeclaration {
                tag: 34,
                cpp_type: "worker_create_operation".to_owned(),
                name: "worker_create_operation".to_owned(),
                fields: vec![OperationField {
                    name: "initializer".to_owned(),
                    cpp_type: "worker_initializer".to_owned(),
                    source_file: "worker.hpp".to_owned(),
                    source_line: 90,
                }],
            },
            OperationDeclaration {
                tag: 36,
                cpp_type: "assert_operation".to_owned(),
                name: "assert_operation".to_owned(),
                fields: vec![OperationField {
                    name: "predicates".to_owned(),
                    cpp_type: "vector<predicate>".to_owned(),
                    source_file: "assert.hpp".to_owned(),
                    source_line: 99,
                }],
            },
        ]);

        assert_eq!(rendered.unsupported_count, 0);
        assert_eq!(rendered.generated_struct_count, 4);
        assert!(rendered.source.contains(
            "pub struct CommitteeMemberUpdateGlobalParametersOperation {\n    pub new_parameters: graphene_protocol::ChainParameters,\n}"
        ));
        assert!(rendered.source.contains(
            "pub struct VestingBalanceCreateOperation {\n    pub policy: graphene_protocol::VestingPolicyInitializer,\n}"
        ));
        assert!(rendered.source.contains(
            "pub struct WorkerCreateOperation {\n    pub initializer: graphene_protocol::WorkerInitializer,\n}"
        ));
        assert!(rendered.source.contains(
            "pub struct AssertOperation {\n    pub predicates: Vec<graphene_protocol::Predicate<graphene_protocol::ObjectId, graphene_protocol::ObjectId>>,\n}"
        ));
    }

    #[test]
    fn maps_s01_operation_option_helpers_to_protocol_safe_types() {
        assert_eq!(
            map_operation_field(&OperationField {
                name: "options".to_owned(),
                cpp_type: "account_options".to_owned(),
                source_file: "account.hpp".to_owned(),
                source_line: 112,
            }),
            OperationFieldClassification::Typed {
                rust_type: "graphene_protocol::AccountOptions<graphene_protocol::ObjectId>"
                    .to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "new_options".to_owned(),
                cpp_type: "optional<account_options>".to_owned(),
                source_file: "account.hpp".to_owned(),
                source_line: 161,
            }),
            OperationFieldClassification::Typed {
                rust_type: "Option<graphene_protocol::AccountOptions<graphene_protocol::ObjectId>>"
                    .to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "common_options".to_owned(),
                cpp_type: "asset_options".to_owned(),
                source_file: "asset_ops.hpp".to_owned(),
                source_line: 215,
            }),
            OperationFieldClassification::Typed {
                rust_type: "graphene_protocol::AssetOptions<graphene_protocol::ObjectId, graphene_protocol::ObjectId>"
                    .to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "bitasset_opts".to_owned(),
                cpp_type: "optional<bitasset_options>".to_owned(),
                source_file: "asset_ops.hpp".to_owned(),
                source_line: 217,
            }),
            OperationFieldClassification::Typed {
                rust_type:
                    "Option<graphene_protocol::BitAssetOptions<graphene_protocol::ObjectId>>"
                        .to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "new_options".to_owned(),
                cpp_type: "bitasset_options".to_owned(),
                source_file: "asset_ops.hpp".to_owned(),
                source_line: 406,
            }),
            OperationFieldClassification::Typed {
                rust_type: "graphene_protocol::BitAssetOptions<graphene_protocol::ObjectId>"
                    .to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "feed".to_owned(),
                cpp_type: "price_feed".to_owned(),
                source_file: "asset_ops.hpp".to_owned(),
                source_line: 475,
            }),
            OperationFieldClassification::Typed {
                rust_type: "graphene_protocol::PriceFeed<graphene_protocol::ObjectId>".to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "on_fill".to_owned(),
                cpp_type: "optional<vector<limit_order_auto_action>>".to_owned(),
                source_file: "market.hpp".to_owned(),
                source_line: 151,
            }),
            OperationFieldClassification::Typed {
                rust_type:
                    "Option<Vec<graphene_protocol::LimitOrderAutoAction<graphene_protocol::ObjectId>>>"
                        .to_owned(),
            }
        );
    }

    #[test]
    fn maps_nested_operation_wrappers_to_chain_local_operation_wrapper() {
        assert_eq!(
            map_operation_field(&OperationField {
                name: "op".to_owned(),
                cpp_type: "op_wrapper".to_owned(),
                source_file: "proposal.hpp".to_owned(),
                source_line: 88,
            }),
            OperationFieldClassification::Typed {
                rust_type: "OperationWrapper".to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "proposed_ops".to_owned(),
                cpp_type: "vector<op_wrapper>".to_owned(),
                source_file: "proposal.hpp".to_owned(),
                source_line: 89,
            }),
            OperationFieldClassification::Typed {
                rust_type: "Vec<OperationWrapper>".to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "argument".to_owned(),
                cpp_type: "operation".to_owned(),
                source_file: "restriction.hpp".to_owned(),
                source_line: 90,
            }),
            OperationFieldClassification::Typed {
                rust_type: "graphene_protocol::RestrictionArgument".to_owned(),
            }
        );

        let rendered = render_operation_structs_module(&[OperationDeclaration {
            tag: 22,
            cpp_type: "proposal_create_operation".to_owned(),
            name: "proposal_create_operation".to_owned(),
            fields: vec![OperationField {
                name: "proposed_ops".to_owned(),
                cpp_type: "vector<op_wrapper>".to_owned(),
                source_file: "proposal.hpp".to_owned(),
                source_line: 91,
            }],
        }])
        .source;

        assert!(rendered.contains(
            "#[derive(Clone, Debug, PartialEq, serde::Deserialize)]\npub struct OperationWrapper {\n    pub op: Operation,\n}\n"
        ));
        assert!(rendered.contains("    pub proposed_ops: Vec<OperationWrapper>,\n"));
    }

    #[test]
    fn operation_extension_raw_fallback_is_scoped_to_approved_operation_fields() {
        let account_create_extensions = OperationField {
            name: "extensions".to_owned(),
            cpp_type: "extension<ext>".to_owned(),
            source_file: "account.hpp".to_owned(),
            source_line: 113,
        };
        assert_eq!(
            map_operation_field_for_operation(
                "account_create_operation",
                &account_create_extensions
            ),
            OperationFieldClassification::ApprovedRawFallback {
                rust_type: "graphene_protocol::RestrictionArgument".to_owned(),
                reason: "approved raw fallback for operation-scoped extension<ext> payload"
                    .to_owned(),
            }
        );

        let asset_update_extensions = OperationField {
            name: "extensions".to_owned(),
            cpp_type: "extension<ext>".to_owned(),
            source_file: "asset_ops.hpp".to_owned(),
            source_line: 377,
        };
        assert_eq!(
            map_operation_field_for_operation("asset_update_operation", &asset_update_extensions),
            OperationFieldClassification::ApprovedRawFallback {
                rust_type: "graphene_protocol::RestrictionArgument".to_owned(),
                reason: "approved raw fallback for operation-scoped extension<ext> payload"
                    .to_owned(),
            }
        );

        for (operation_name, cpp_type, source_line) in [
            ("asset_publish_feed_operation", "extension<ext>", 476),
            (
                "asset_claim_fees_operation",
                "extension<additional_options_type>",
                1043,
            ),
            ("credit_offer_accept_operation", "extension<ext>", 83),
        ] {
            assert_eq!(
                map_operation_field_for_operation(
                    operation_name,
                    &OperationField {
                        name: "extensions".to_owned(),
                        cpp_type: cpp_type.to_owned(),
                        source_file: "asset_ops.hpp".to_owned(),
                        source_line,
                    }
                ),
                OperationFieldClassification::ApprovedRawFallback {
                    rust_type: "graphene_protocol::RestrictionArgument".to_owned(),
                    reason: if cpp_type == "extension<ext>" {
                        "approved raw fallback for operation-scoped extension<ext> payload"
                            .to_owned()
                    } else {
                        "approved raw fallback for extension/static-variant payload".to_owned()
                    },
                }
            );
        }

        assert_eq!(
            map_operation_field_for_operation(
                "unrelated_operation",
                &OperationField {
                    name: "extensions".to_owned(),
                    cpp_type: "extension<ext>".to_owned(),
                    source_file: "operations.hpp".to_owned(),
                    source_line: 999,
                }
            ),
            OperationFieldClassification::Unsupported {
                reason:
                    "raw fallback is restricted to approved extension/static-variant payload fields"
                        .to_owned(),
            }
        );
    }

    #[test]
    fn operation_mapper_types_htlc_and_confidential_boundaries() {
        for (operation_name, field_name) in [
            ("htlc_create_operation", "preimage_hash"),
            ("htlc_redeemed_operation", "htlc_preimage_hash"),
            ("htlc_refund_operation", "htlc_preimage_hash"),
        ] {
            assert_eq!(
                map_operation_field_for_operation(
                    operation_name,
                    &OperationField {
                        name: field_name.to_owned(),
                        cpp_type: "htlc_hash".to_owned(),
                        source_file: "m003.hpp".to_owned(),
                        source_line: 1,
                    },
                ),
                OperationFieldClassification::Typed {
                    rust_type: "graphene_protocol::HtlcHash".to_owned(),
                },
                "{operation_name}.{field_name} should map htlc_hash to the shared protocol primitive"
            );
        }

        assert_eq!(
            map_operation_field_for_operation(
                "htlc_create_operation",
                &OperationField {
                    name: "extensions".to_owned(),
                    cpp_type: "extension<additional_options_type>".to_owned(),
                    source_file: "m003.hpp".to_owned(),
                    source_line: 1,
                },
            ),
            OperationFieldClassification::ApprovedRawFallback {
                rust_type: "graphene_protocol::RestrictionArgument".to_owned(),
                reason: "approved raw fallback for extension/static-variant payload".to_owned(),
            },
            "htlc_create_operation.extensions should use the approved raw fallback boundary"
        );

        for (operation_name, field_name, cpp_type, rust_type) in [
            (
                "transfer_to_blind_operation",
                "blinding_factor",
                "blind_factor_type",
                "graphene_protocol::BlindFactor",
            ),
            (
                "blind_transfer_operation",
                "inputs",
                "vector<blind_input>",
                "Vec<graphene_protocol::BlindInput>",
            ),
            (
                "blind_transfer_operation",
                "outputs",
                "vector<blind_output>",
                "Vec<graphene_protocol::BlindOutput>",
            ),
            (
                "transfer_from_blind_operation",
                "inputs",
                "vector<blind_input>",
                "Vec<graphene_protocol::BlindInput>",
            ),
        ] {
            assert_eq!(
                map_operation_field_for_operation(
                    operation_name,
                    &OperationField {
                        name: field_name.to_owned(),
                        cpp_type: cpp_type.to_owned(),
                        source_file: "confidential.hpp".to_owned(),
                        source_line: 1,
                    },
                ),
                OperationFieldClassification::Typed {
                    rust_type: rust_type.to_owned(),
                },
                "{operation_name}.{field_name} should map {cpp_type} to {rust_type}"
            );
        }
    }

    #[test]
    fn operation_mapper_distinguishes_raw_fallback_from_unsupported() {
        assert_eq!(
            map_operation_field(&OperationField {
                name: "extensions".to_owned(),
                cpp_type: "extensions_type".to_owned(),
                source_file: "operations.hpp".to_owned(),
                source_line: 9,
            }),
            OperationFieldClassification::ApprovedRawFallback {
                rust_type: "Vec<graphene_protocol::RestrictionArgument>".to_owned(),
                reason: "approved raw fallback for extension/static-variant payload".to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "mystery".to_owned(),
                cpp_type: "flat_map<account_id_type, mystery_type>".to_owned(),
                source_file: "operations.hpp".to_owned(),
                source_line: 10,
            }),
            OperationFieldClassification::Unsupported {
                reason: "no protocol-safe operation mapping for C++ type flat_map<account_id_type, mystery_type>".to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "unsupported_extensions".to_owned(),
                cpp_type: "flat_set<future_extensions>".to_owned(),
                source_file: "operations.hpp".to_owned(),
                source_line: 11,
            }),
            OperationFieldClassification::Unsupported {
                reason:
                    "raw fallback is restricted to approved extension/static-variant payload fields"
                        .to_owned(),
            }
        );
        assert_eq!(
            map_operation_field(&OperationField {
                name: "mystery_variant".to_owned(),
                cpp_type: "static_variant<known_type, unknown_type>".to_owned(),
                source_file: "operations.hpp".to_owned(),
                source_line: 12,
            }),
            OperationFieldClassification::Unsupported {
                reason: "no protocol-safe operation mapping for C++ type static_variant<known_type, unknown_type>"
                    .to_owned(),
            }
        );
    }

    #[test]
    fn renders_operation_structs_in_variant_order_with_field_order_and_numeric_helpers() {
        let declarations = vec![
            OperationDeclaration {
                tag: 1,
                cpp_type: "graphene::chain::zero_field_operation".to_owned(),
                name: "zero_field_operation".to_owned(),
                fields: vec![],
            },
            OperationDeclaration {
                tag: 0,
                cpp_type: "transfer_operation".to_owned(),
                name: "transfer_operation".to_owned(),
                fields: vec![
                    OperationField {
                        name: "from".to_owned(),
                        cpp_type: "account_id_type".to_owned(),
                        source_file: "operations.hpp".to_owned(),
                        source_line: 5,
                    },
                    OperationField {
                        name: "fee".to_owned(),
                        cpp_type: "asset".to_owned(),
                        source_file: "operations.hpp".to_owned(),
                        source_line: 6,
                    },
                    OperationField {
                        name: "expiration".to_owned(),
                        cpp_type: "uint64_t".to_owned(),
                        source_file: "operations.hpp".to_owned(),
                        source_line: 7,
                    },
                ],
            },
        ];

        let rendered = render_operation_structs_module(&declarations).source;

        assert!(rendered.starts_with("// @generated by graphene-codegen; do not edit by hand.\n"));
        assert!(
            rendered.find("pub struct TransferOperation").unwrap()
                < rendered.find("pub struct ZeroFieldOperation").unwrap()
        );
        assert!(rendered.contains(
            "#[derive(Clone, Debug, PartialEq, serde::Deserialize)]\npub struct TransferOperation {"
        ));
        assert!(rendered.contains("    pub from: graphene_protocol::ObjectId,\n    pub fee: graphene_protocol::Asset<graphene_protocol::ObjectId>,\n    #[serde(deserialize_with = \"graphene_protocol::u64_from_number_or_string\")]\n    pub expiration: u64,"));
        assert!(rendered.contains("pub struct ZeroFieldOperation {}\n"));
    }

    #[test]
    fn operation_rendering_skips_unsupported_structs_and_reports_source_metadata() {
        let declarations = vec![OperationDeclaration {
            tag: 37,
            cpp_type: "balance_claim_operation".to_owned(),
            name: "balance_claim_operation".to_owned(),
            fields: vec![
                OperationField {
                    name: "owner".to_owned(),
                    cpp_type: "address".to_owned(),
                    source_file: "balance.hpp".to_owned(),
                    source_line: 12,
                },
                OperationField {
                    name: "extensions".to_owned(),
                    cpp_type: "extensions_type".to_owned(),
                    source_file: "balance.hpp".to_owned(),
                    source_line: 13,
                },
            ],
        }];

        let rendered = render_operation_structs_module(&declarations);
        assert!(!rendered.source.contains("pub struct BalanceClaimOperation"));
        assert_eq!(rendered.generated_struct_count, 0);
        assert_eq!(rendered.unsupported_count, 1);
        assert_eq!(rendered.raw_fallback_count, 1);
        assert_eq!(
            rendered.report_rows,
            vec![
                OperationModelReportRow {
                    operation: "balance_claim_operation".to_owned(),
                    tag: 37,
                    field: "owner".to_owned(),
                    cpp_type: "address".to_owned(),
                    source_file: "balance.hpp".to_owned(),
                    source_line: 12,
                    classification: "unsupported".to_owned(),
                    reason: "no protocol-safe operation mapping for C++ type address".to_owned(),
                },
                OperationModelReportRow {
                    operation: "balance_claim_operation".to_owned(),
                    tag: 37,
                    field: "extensions".to_owned(),
                    cpp_type: "extensions_type".to_owned(),
                    source_file: "balance.hpp".to_owned(),
                    source_line: 13,
                    classification: "approved_raw_fallback".to_owned(),
                    reason: "approved raw fallback for extension/static-variant payload".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn renders_operation_model_skips_report_with_totals_and_rows() {
        let report = render_operation_model_skips_report(
            4,
            2,
            &[
                OperationModelReportRow {
                    operation: "balance_claim_operation".to_owned(),
                    tag: 37,
                    field: "owner".to_owned(),
                    cpp_type: "address".to_owned(),
                    source_file: "balance.hpp".to_owned(),
                    source_line: 12,
                    classification: "unsupported".to_owned(),
                    reason: "no protocol-safe operation mapping for C++ type address".to_owned(),
                },
                OperationModelReportRow {
                    operation: "transfer_operation".to_owned(),
                    tag: 0,
                    field: "extensions".to_owned(),
                    cpp_type: "extensions_type".to_owned(),
                    source_file: "operations.hpp".to_owned(),
                    source_line: 9,
                    classification: "approved_raw_fallback".to_owned(),
                    reason: "approved raw fallback for extension/static-variant payload".to_owned(),
                },
            ],
        );

        assert_eq!(
            report,
            concat!(
                "# Operation model coverage report\n",
                "\n",
                "@generated by graphene-codegen; do not edit by hand.\n",
                "\n",
                "- Total operations: 4\n",
                "- Generated structs: 2\n",
                "- Skipped/unsupported rows: 1\n",
                "- Approved raw fallback rows: 1\n",
                "\n",
                "| Operation | Tag | Field | Normalized C++ type | Source | Classification | Reason |\n",
                "|---|---:|---|---|---|---|---|\n",
                "| balance_claim_operation | 37 | owner | `address` | balance.hpp:12 | unsupported | no protocol-safe operation mapping for C++ type address |\n",
                "| transfer_operation | 0 | extensions | `extensions_type` | operations.hpp:9 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |\n",
            )
        );
    }

    #[test]
    fn parses_protocol_object_families() {
        let source = r#"
            GRAPHENE_DEFINE_IDS(protocol, protocol_ids, /*protocol objects are not prefixed*/,
                                /* 1.0.x  */ (null)
                                /* 1.1.x  */ (base)
                                /* 1.2.x  */ (account)
                                /* 1.3.x  */ (asset)
                               )
        "#;

        assert_eq!(
            parse_object_families(source).expect("source should parse"),
            vec![
                ObjectFamily {
                    object_space: 1,
                    type_id: 0,
                    name: "null".to_owned(),
                },
                ObjectFamily {
                    object_space: 1,
                    type_id: 1,
                    name: "base".to_owned(),
                },
                ObjectFamily {
                    object_space: 1,
                    type_id: 2,
                    name: "account".to_owned(),
                },
                ObjectFamily {
                    object_space: 1,
                    type_id: 3,
                    name: "asset".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn parses_implementation_object_families() {
        let source = r#"
            GRAPHENE_DEFINE_IDS(chain, implementation_ids, impl_,
                                /* 2.0.x  */ (global_property)
                                /* 2.1.x  */ (dynamic_global_property)
                                /* 2.2.x  */ (reserved0)
                                /* 2.3.x  */ (asset_dynamic_data)
                                /* 2.4.x  */ (asset_bitasset_data)
                                /* 2.5.x  */ (account_balance)
                               )
        "#;

        assert_eq!(
            parse_object_families(source).expect("source should parse"),
            vec![
                ObjectFamily {
                    object_space: 2,
                    type_id: 0,
                    name: "global_property".to_owned(),
                },
                ObjectFamily {
                    object_space: 2,
                    type_id: 1,
                    name: "dynamic_global_property".to_owned(),
                },
                ObjectFamily {
                    object_space: 2,
                    type_id: 2,
                    name: "reserved0".to_owned(),
                },
                ObjectFamily {
                    object_space: 2,
                    type_id: 3,
                    name: "asset_dynamic_data".to_owned(),
                },
                ObjectFamily {
                    object_space: 2,
                    type_id: 4,
                    name: "asset_bitasset_data".to_owned(),
                },
                ObjectFamily {
                    object_space: 2,
                    type_id: 5,
                    name: "account_balance".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn parses_family_names_before_line_comments_with_commas() {
        let source = r#"
            GRAPHENE_DEFINE_IDS(chain, implementation_ids, impl_,
                                /* 2.0.x  */ (global_property)
                                /* 2.1.x  */ (dynamic_global_property)
                                /* 2.2.x  */ (reserved0) // unused, but kept for compatibility
                                /* 2.3.x  */ (asset_dynamic_data)
                               )
        "#;

        assert_eq!(
            parse_object_families(source).expect("source should parse"),
            vec![
                ObjectFamily {
                    object_space: 2,
                    type_id: 0,
                    name: "global_property".to_owned(),
                },
                ObjectFamily {
                    object_space: 2,
                    type_id: 1,
                    name: "dynamic_global_property".to_owned(),
                },
                ObjectFamily {
                    object_space: 2,
                    type_id: 2,
                    name: "reserved0".to_owned(),
                },
                ObjectFamily {
                    object_space: 2,
                    type_id: 3,
                    name: "asset_dynamic_data".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn falls_back_to_sequence_order_when_comments_are_missing() {
        let source = r#"
            GRAPHENE_DEFINE_IDS(protocol, protocol_ids, prefix_,
                                (account)
                                (asset)
                               )
        "#;

        assert_eq!(
            parse_object_families(source).expect("source should parse"),
            vec![
                ObjectFamily {
                    object_space: 1,
                    type_id: 0,
                    name: "account".to_owned(),
                },
                ObjectFamily {
                    object_space: 1,
                    type_id: 1,
                    name: "asset".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn validates_comments_against_declaration_order_without_trusting_them() {
        let source = r#"
            GRAPHENE_DEFINE_IDS(protocol, protocol_ids, prefix_,
                                /* 1.99.x */ (account)
                               )
        "#;

        let error = parse_object_families(source).expect_err("comment/order mismatch should fail");
        assert!(error.contains("disagrees with declaration order"));
    }

    #[test]
    fn renders_protocol_object_id_module_with_explicit_object_space() {
        let source = render_object_id_module(&ObjectFamily {
            object_space: 1,
            type_id: 2,
            name: "account".to_owned(),
        });

        assert_eq!(
            source,
            concat!(
                "// @generated by graphene-codegen; do not edit by hand.\n",
                "// Source family: account (1.2.x)\n",
                "\n",
                "graphene_protocol::define_object_id_type! {\n",
                "    name: \"account\",\n",
                "    object_space: 1,\n",
                "    type_id: 2,\n",
                "}\n",
            )
        );
    }

    #[test]
    fn renders_implementation_object_id_module_with_explicit_object_space() {
        let source = render_object_id_module(&ObjectFamily {
            object_space: 2,
            type_id: 5,
            name: "account_balance".to_owned(),
        });

        assert_eq!(
            source,
            concat!(
                "// @generated by graphene-codegen; do not edit by hand.\n",
                "// Source family: account_balance (2.5.x)\n",
                "\n",
                "graphene_protocol::define_object_id_type! {\n",
                "    name: \"account_balance\",\n",
                "    object_space: 2,\n",
                "    type_id: 5,\n",
                "}\n",
            )
        );
    }

    #[test]
    fn renders_types_mod_for_generated_family_modules() {
        let source = render_types_mod(&[
            ObjectFamily {
                object_space: 1,
                type_id: 2,
                name: "account".to_owned(),
            },
            ObjectFamily {
                object_space: 2,
                type_id: 5,
                name: "account_balance".to_owned(),
            },
        ]);

        assert_eq!(
            source,
            concat!(
                "// @generated by graphene-codegen; do not edit by hand.\n",
                "\n",
                "pub mod account;\n",
                "pub mod account_balance;\n",
            )
        );
    }

    #[test]
    fn parses_class_body_after_forward_declaration() {
        let header = r#"
            class asset_bitasset_data_object;

            class asset_bitasset_data_object : public abstract_object<asset_bitasset_data_object,
                                                implementation_ids, impl_asset_bitasset_data_object_type>
            {
               public:
                  asset_id_type asset_id;
                  share_type force_settled_volume;
            };
        "#;
        let reflected_fields = ["asset_id", "force_settled_volume"];

        let cpp_fields =
            parse_reflected_class_fields(header, "asset_bitasset_data_object", &reflected_fields)
                .expect("full class body should be parsed after forward declaration");

        assert_eq!(
            cpp_fields,
            vec![
                CppField {
                    name: "asset_id".to_owned(),
                    cpp_type: "asset_id_type".to_owned(),
                },
                CppField {
                    name: "force_settled_volume".to_owned(),
                    cpp_type: "share_type".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn generic_object_mapping_keeps_operation_and_transaction_wrappers_raw_compatible() {
        let fields = vec![
            CppField {
                name: "op".to_owned(),
                cpp_type: "operation".to_owned(),
            },
            CppField {
                name: "result".to_owned(),
                cpp_type: "operation_result".to_owned(),
            },
            CppField {
                name: "proposed_transaction".to_owned(),
                cpp_type: "transaction".to_owned(),
            },
            CppField {
                name: "trx".to_owned(),
                cpp_type: "signed_transaction".to_owned(),
            },
        ];

        assert_eq!(
            map_fields_to_rust(&fields).expect("generic object fields should map"),
            vec![
                RustField {
                    name: "op".to_owned(),
                    rust_type: "graphene_protocol::Operation".to_owned(),
                },
                RustField {
                    name: "result".to_owned(),
                    rust_type: "graphene_protocol::OperationResult".to_owned(),
                },
                RustField {
                    name: "proposed_transaction".to_owned(),
                    rust_type: "graphene_protocol::Transaction".to_owned(),
                },
                RustField {
                    name: "trx".to_owned(),
                    rust_type: "graphene_protocol::SignedTransaction".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn bitshares_object_mapping_uses_chain_local_transaction_models() {
        let fields = vec![
            CppField {
                name: "op".to_owned(),
                cpp_type: "operation".to_owned(),
            },
            CppField {
                name: "result".to_owned(),
                cpp_type: "operation_result".to_owned(),
            },
            CppField {
                name: "proposed_transaction".to_owned(),
                cpp_type: "transaction".to_owned(),
            },
            CppField {
                name: "trx".to_owned(),
                cpp_type: "signed_transaction".to_owned(),
            },
        ];

        assert_eq!(
            map_fields_to_rust_for_chain("bitshares", &fields)
                .expect("BitShares object fields should map"),
            vec![
                RustField {
                    name: "op".to_owned(),
                    rust_type: "crate::operations::Operation".to_owned(),
                },
                RustField {
                    name: "result".to_owned(),
                    rust_type: "crate::transaction::OperationResult".to_owned(),
                },
                RustField {
                    name: "proposed_transaction".to_owned(),
                    rust_type: "crate::transaction::Transaction".to_owned(),
                },
                RustField {
                    name: "trx".to_owned(),
                    rust_type: "crate::transaction::SignedTransaction".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn maps_basic_graphene_cpp_types_to_rust_types() {
        assert_eq!(map_cpp_type_to_rust("bool"), Some("bool".to_owned()));
        assert_eq!(map_cpp_type_to_rust("uint16_t"), Some("u16".to_owned()));
        assert_eq!(map_cpp_type_to_rust("share_type"), Some("i64".to_owned()));
        assert_eq!(
            map_cpp_type_to_rust("asset"),
            Some("graphene_protocol::Asset<crate::types::asset::Id>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("block_id_type"),
            Some("String".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("chain_id_type"),
            Some("String".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("fc::ecc::commitment_type"),
            Some("String".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("public_key_type"),
            Some("String".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("fc::uint128_t"),
            Some("u128".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("vote_id_type"),
            Some("String".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("vesting_policy"),
            Some("graphene_protocol::VestingPolicy".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("vesting_balance_type"),
            Some("String".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("worker_type"),
            Some("graphene_protocol::WorkerType<crate::types::vesting_balance::Id>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("ticket_type"),
            Some("String".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("ticket_status"),
            Some("String".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("authority"),
            Some("graphene_protocol::Authority<crate::types::account::Id>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("chain_parameters"),
            Some("graphene_protocol::ChainParameters".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("immutable_chain_parameters"),
            Some("graphene_protocol::ImmutableChainParameters".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("limit_order_auto_action"),
            Some("graphene_protocol::LimitOrderAutoAction<crate::types::asset::Id>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("transfer_info"),
            Some(
                "graphene_protocol::HtlcTransfer<crate::types::account::Id, crate::types::asset::Id>"
                    .to_owned()
            )
        );
        assert_eq!(
            map_cpp_type_to_rust("condition_info"),
            Some("graphene_protocol::HtlcConditions".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("fc::optional<memo_data>"),
            Some("Option<graphene_protocol::MemoData>".to_owned())
        );
        assert_eq!(map_cpp_type_to_rust("address"), Some("String".to_owned()));
        assert_eq!(
            map_cpp_type_to_rust("account_options"),
            Some("graphene_protocol::AccountOptions<crate::types::account::Id>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("special_authority"),
            Some("graphene_protocol::SpecialAuthority<crate::types::asset::Id>".to_owned())
        );
        assert_eq!(map_cpp_type_to_rust("unsigned_int"), Some("u64".to_owned()));
        assert_eq!(
            map_cpp_type_to_rust("asset_options"),
            Some(
                "graphene_protocol::AssetOptions<crate::types::account::Id, crate::types::asset::Id>"
                    .to_owned()
            )
        );
        assert_eq!(
            map_cpp_type_to_rust("bitasset_options"),
            Some("graphene_protocol::BitAssetOptions<crate::types::asset::Id>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("budget_record"),
            Some("graphene_protocol::BudgetRecord".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("operation"),
            Some("graphene_protocol::Operation".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("operation_result"),
            Some("graphene_protocol::OperationResult".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("transaction"),
            Some("graphene_protocol::Transaction".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("signed_transaction"),
            Some("graphene_protocol::SignedTransaction".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("transaction_id_type"),
            Some("String".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("price_feed_with_icr"),
            Some("graphene_protocol::PriceFeedWithIcr<crate::types::asset::Id>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("pair<time_point_sec,price_feed_with_icr>"),
            Some(
                "(String, graphene_protocol::PriceFeedWithIcr<crate::types::asset::Id>)".to_owned()
            )
        );
        assert_eq!(
            map_cpp_type_to_rust("flat_map<account_id_type, pair<time_point_sec,price_feed_with_icr>>"),
            Some(
                "Vec<(crate::types::account::Id, (String, graphene_protocol::PriceFeedWithIcr<crate::types::asset::Id>))>"
                    .to_owned()
            )
        );
        assert_eq!(
            map_cpp_type_to_rust("restriction"),
            Some("graphene_protocol::Restriction".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("flat_map<uint16_t, restriction>"),
            Some("Vec<(u16, graphene_protocol::Restriction)>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("optional<linear_vesting_policy>"),
            Some("Option<graphene_protocol::LinearVestingPolicy>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("price"),
            Some("graphene_protocol::Price<crate::types::asset::Id>".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("account_id_type"),
            Some("crate::types::account::Id".to_owned())
        );
        assert_eq!(
            map_cpp_type_to_rust("optional< flat_set<asset_id_type> >"),
            Some("Option<Vec<crate::types::asset::Id>>".to_owned())
        );
    }

    #[test]
    fn parses_and_maps_account_balance_object_fields() {
        let header = r#"
            class account_balance_object : public abstract_object<account_balance_object,
                                                implementation_ids, impl_account_balance_object_type>
            {
               public:
                  account_id_type   owner;
                  asset_id_type     asset_type;
                  share_type        balance;
                  bool              maintenance_flag = false;

                  asset get_balance()const { return asset(balance, asset_type); }
                  void  adjust_balance(const asset& delta);
            };
        "#;
        let reflected_fields = ["owner", "asset_type", "balance", "maintenance_flag"];

        let cpp_fields =
            parse_reflected_class_fields(header, "account_balance_object", &reflected_fields)
                .expect("account balance fields should parse");

        assert_eq!(
            cpp_fields,
            vec![
                CppField {
                    name: "owner".to_owned(),
                    cpp_type: "account_id_type".to_owned(),
                },
                CppField {
                    name: "asset_type".to_owned(),
                    cpp_type: "asset_id_type".to_owned(),
                },
                CppField {
                    name: "balance".to_owned(),
                    cpp_type: "share_type".to_owned(),
                },
                CppField {
                    name: "maintenance_flag".to_owned(),
                    cpp_type: "bool".to_owned(),
                },
            ]
        );

        assert_eq!(
            map_fields_to_rust(&cpp_fields).expect("account balance fields should map"),
            vec![
                RustField {
                    name: "owner".to_owned(),
                    rust_type: "crate::types::account::Id".to_owned(),
                },
                RustField {
                    name: "asset_type".to_owned(),
                    rust_type: "crate::types::asset::Id".to_owned(),
                },
                RustField {
                    name: "balance".to_owned(),
                    rust_type: "i64".to_owned(),
                },
                RustField {
                    name: "maintenance_flag".to_owned(),
                    rust_type: "bool".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn parses_field_with_braced_initializer() {
        let header = r#"
            class witness_object : public abstract_object<witness_object, protocol_ids, witness_object_type>
            {
               public:
                  vote_id_type     vote_id { vote_id_type::witness };
            };
        "#;

        assert_eq!(
            parse_reflected_class_fields(header, "witness_object", &["vote_id"])
                .expect("witness vote_id field should parse"),
            vec![CppField {
                name: "vote_id".to_owned(),
                cpp_type: "vote_id_type".to_owned(),
            }]
        );
    }

    #[test]
    fn parses_fields_after_inline_method_body() {
        let header = r#"
            class account_statistics_object : public abstract_object<account_statistics_object,
                                               implementation_ids, impl_account_statistics_object_type>
            {
               public:
                  bool is_voting = false;

                  inline bool has_some_core_voting() const
                  {
                     return is_voting;
                  }

                  share_type lifetime_fees_paid;
                  share_type pending_fees;
            };
        "#;

        assert_eq!(
            parse_reflected_class_fields(
                header,
                "account_statistics_object",
                &["is_voting", "lifetime_fees_paid", "pending_fees"]
            )
            .expect("fields after inline method should parse"),
            vec![
                CppField {
                    name: "is_voting".to_owned(),
                    cpp_type: "bool".to_owned(),
                },
                CppField {
                    name: "lifetime_fees_paid".to_owned(),
                    cpp_type: "share_type".to_owned(),
                },
                CppField {
                    name: "pending_fees".to_owned(),
                    cpp_type: "share_type".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn parses_fc_reflect_derived_field_order() {
        let source = r#"
            FC_REFLECT_DERIVED_NO_TYPENAME( graphene::chain::account_balance_object,
                                (graphene::db::object),
                                (owner)(asset_type)(balance)(maintenance_flag) )
        "#;

        assert_eq!(
            parse_reflected_objects(source).expect("reflection should parse"),
            vec![ReflectedObject {
                cpp_type: "graphene::chain::account_balance_object".to_owned(),
                fields: vec![
                    "owner".to_owned(),
                    "asset_type".to_owned(),
                    "balance".to_owned(),
                    "maintenance_flag".to_owned(),
                ],
            }]
        );
    }

    #[test]
    fn maps_account_balance_object_from_fc_reflection_and_header_declarations() {
        let header = r#"
            class account_balance_object : public abstract_object<account_balance_object,
                                                implementation_ids, impl_account_balance_object_type>
            {
               public:
                  account_id_type   owner;
                  asset_id_type     asset_type;
                  share_type        balance;
                  bool              maintenance_flag = false;

                  asset get_balance()const { return asset(balance, asset_type); }
                  void  adjust_balance(const asset& delta);
            };
        "#;
        let reflection = r#"
            FC_REFLECT_DERIVED_NO_TYPENAME( graphene::chain::account_balance_object,
                                (graphene::db::object),
                                (owner)(asset_type)(balance)(maintenance_flag) )
        "#;

        let reflected = parse_reflected_objects(reflection)
            .expect("reflection should parse")
            .pop()
            .expect("one reflected object");
        let reflected_fields = reflected
            .fields
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let cpp_fields =
            parse_reflected_class_fields(header, "account_balance_object", &reflected_fields)
                .expect("fields should parse");

        assert_eq!(
            map_fields_to_rust(&cpp_fields).expect("fields should map"),
            vec![
                RustField {
                    name: "owner".to_owned(),
                    rust_type: "crate::types::account::Id".to_owned(),
                },
                RustField {
                    name: "asset_type".to_owned(),
                    rust_type: "crate::types::asset::Id".to_owned(),
                },
                RustField {
                    name: "balance".to_owned(),
                    rust_type: "i64".to_owned(),
                },
                RustField {
                    name: "maintenance_flag".to_owned(),
                    rust_type: "bool".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn renders_object_struct_from_mapped_account_balance_fields() {
        let fields = vec![
            RustField {
                name: "owner".to_owned(),
                rust_type: "crate::types::account::Id".to_owned(),
            },
            RustField {
                name: "asset_type".to_owned(),
                rust_type: "crate::types::asset::Id".to_owned(),
            },
            RustField {
                name: "balance".to_owned(),
                rust_type: "i64".to_owned(),
            },
            RustField {
                name: "maintenance_flag".to_owned(),
                rust_type: "bool".to_owned(),
            },
        ];

        assert_eq!(
            render_object_struct(&fields),
            concat!(
                "#[derive(Clone, Debug, PartialEq, serde::Deserialize)]\n",
                "pub struct Object {\n",
                "    pub id: Id,\n",
                "    pub owner: crate::types::account::Id,\n",
                "    pub asset_type: crate::types::asset::Id,\n",
                "    #[serde(deserialize_with = \"graphene_protocol::i64_from_number_or_string\")]\n",
                "    pub balance: i64,\n",
                "    pub maintenance_flag: bool,\n",
                "}\n",
            )
        );
    }
}
