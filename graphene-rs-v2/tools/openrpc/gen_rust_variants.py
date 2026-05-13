#!/usr/bin/env python3
"""Generate Rust enums for Graphene/FC static_variant schemas in an OpenRPC spec.

The input spec is expected to model static variants as JSON Schema `oneOf`
alternatives where each alternative is a two-item array `[index, payload]`:

    {
      "oneOf": [
        {
          "type": "array",
          "prefixItems": [
            { "const": 0, "description": "transfer_operation" },
            { "$ref": "#/components/schemas/transfer_operation" }
          ],
          "minItems": 2,
          "maxItems": 2
        }
      ]
    }

Typify can compile the payload structs, but this Graphene wire shape needs a
custom enum with manual Serialize/Deserialize. This script emits that enum for
every matching schema plus a names file that downstream typify integration can
use to remove colliding typify-generated placeholder enums.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def snake_to_pascal(name: str) -> str:
    """Match typify's snake_case -> PascalCase naming for schema identifiers."""
    return "".join(p[:1].upper() + p[1:] for p in name.split("_") if p)


_TRIM_SUFFIXES = (
    "::fee_params_t",
    "_operation",
    "_initializer",
    "_authority",
    "_policy",
    "_argument_type",
)


def _trim_suffix(value: str) -> str:
    for suffix in _TRIM_SUFFIXES:
        if value.endswith(suffix) and len(value) > len(suffix) + 1:
            return value[: -len(suffix)]
    return value


def variant_ident(alt_desc: str, used: set[str]) -> str:
    """Convert a C++ alternative description into a unique Rust variant name."""
    import re as _re

    name = alt_desc.split("::")[-1]
    name = _re.sub(r"[^A-Za-z0-9_]", "_", name)
    name = _trim_suffix(name)
    ident = snake_to_pascal(name) or "V"
    if not ident[0].isalpha():
        ident = "V" + ident
    base = ident
    index = 2
    while ident in used:
        ident = f"{base}{index}"
        index += 1
    used.add(ident)
    return ident


def is_variant_schema(schema: dict) -> bool:
    one_of = schema.get("oneOf")
    if not isinstance(one_of, list) or len(one_of) < 1:
        return False
    for alt in one_of:
        if not isinstance(alt, dict) or alt.get("type") != "array":
            return False
        prefix_items = alt.get("prefixItems")
        if not isinstance(prefix_items, list) or len(prefix_items) != 2:
            return False
        if "const" not in prefix_items[0]:
            return False
    return True


_INT_FORMAT_MAP = {
    "int8": "i8",
    "int16": "i16",
    "int32": "i32",
    "int64": "i64",
    "uint8": "u8",
    "uint16": "u16",
    "uint32": "u32",
    "uint64": "u64",
}


def payload_rust_type(payload: dict) -> str:
    """Translate one payload schema fragment into a Rust type expression."""
    if "$ref" in payload:
        target = payload["$ref"].split("/")[-1]
        return snake_to_pascal(target)

    ty = payload.get("type")
    if isinstance(ty, list):
        non_null = [t for t in ty if t != "null"]
        if len(non_null) == 1:
            inner = payload_rust_type({**payload, "type": non_null[0]})
            return f"::std::option::Option<{inner}>"
        return "::serde_json::Value"

    if ty == "boolean":
        return "bool"
    if ty == "string":
        if payload.get("format") == "date-time":
            return "::chrono::DateTime<::chrono::Utc>"
        return "::std::string::String"
    if ty == "integer":
        return _INT_FORMAT_MAP.get(payload.get("format", ""), "i64")
    if ty == "number":
        return "f32" if payload.get("format") == "float" else "f64"
    if ty == "array":
        items = payload.get("items", {})
        return f"::std::vec::Vec<{payload_rust_type(items)}>"
    if ty == "null":
        return "()"
    if ty == "object":
        additional_properties = payload.get("additionalProperties")
        if isinstance(additional_properties, dict):
            value_type = payload_rust_type(additional_properties)
            return f"::std::collections::HashMap<::std::string::String, {value_type}>"
    return "::serde_json::Value"


def collect_variants(spec: dict) -> list[dict]:
    schemas = spec["components"]["schemas"]
    variants: list[dict] = []
    for schema_name, schema in schemas.items():
        if not is_variant_schema(schema):
            continue
        used: set[str] = set()
        alternatives: list[dict] = []
        for alt in schema["oneOf"]:
            prefix_items = alt["prefixItems"]
            index = prefix_items[0]["const"]
            description = prefix_items[0].get("description", f"v{index}")
            payload = prefix_items[1]
            alternatives.append(
                {
                    "index": index,
                    "rust_variant": variant_ident(description, used),
                    "rust_type": payload_rust_type(payload),
                    "alt_cpp": description,
                }
            )
        variants.append(
            {
                "schema_name": schema_name,
                "rust_type": snake_to_pascal(schema_name),
                "cpp_type": schema.get("x-cpp-type", ""),
                "alts": alternatives,
            }
        )
    return variants


def emit(variants: list[dict], source_label: str) -> str:
    out: list[str] = [
        "// AUTO-GENERATED by graphene-rs-v2/tools/openrpc/gen_rust_variants.py; do not edit by hand.",
        f"// Source: {source_label} (components.schemas)",
        "//",
        "// Each enum corresponds to an fc::static_variant<...> in the C++ protocol.",
        "// On the wire it is the 2-element JSON array `[index, payload]`.",
        "",
        "use serde::de::{Deserialize, Deserializer, Error as _DeError};",
        "use serde::ser::{Serialize, SerializeTuple, Serializer};",
        "",
    ]
    for variant in variants:
        ty = variant["rust_type"]
        cpp = variant["cpp_type"] or variant["schema_name"]
        out.append(f"/// `{cpp}` — {len(variant['alts'])} alternatives.")
        out.append("///")
        out.append(f"/// Schema: `components.schemas.{variant['schema_name']}`.")
        out.append("#[derive(Clone, Debug)]")
        out.append(f"pub enum {ty} {{")
        for alt in variant["alts"]:
            out.append(f"    /// Wire index `{alt['index']}` — C++ `{alt['alt_cpp']}`")
            out.append(f"    {alt['rust_variant']}({alt['rust_type']}),")
        out.append("}")
        out.append("")

        out.append(f"impl Serialize for {ty} {{")
        out.append("    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {")
        out.append("        let mut tuple = ser.serialize_tuple(2)?;")
        out.append("        match self {")
        for alt in variant["alts"]:
            out.append(
                f"            {ty}::{alt['rust_variant']}(value) => {{ "
                f"tuple.serialize_element(&{alt['index']}u32)?; "
                "tuple.serialize_element(value)?; }"
            )
        out.append("        }")
        out.append("        tuple.end()")
        out.append("    }")
        out.append("}")
        out.append("")

        out.append(f"impl<'de> Deserialize<'de> for {ty} {{")
        out.append("    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {")
        out.append("        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;")
        out.append("        match index {")
        for alt in variant["alts"]:
            out.append(
                f"            {alt['index']} => ::serde_json::from_value(payload)"
                f".map({ty}::{alt['rust_variant']})"
                ".map_err(__D::Error::custom),"
            )
        out.append(
            f"            other => Err(__D::Error::custom(format!(\"unknown {ty} variant index: {{}}\", other))),"
        )
        out.append("        }")
        out.append("    }")
        out.append("}")
        out.append("")
    return "\n".join(out)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", required=True, type=Path, help="OpenRPC spec JSON path")
    parser.add_argument("--out-rs", required=True, type=Path, help="Rust file to write")
    parser.add_argument(
        "--out-names",
        required=True,
        type=Path,
        help="newline-separated generated enum type names",
    )
    parser.add_argument(
        "--source-label",
        default=None,
        help="label to place in generated file header; defaults to --spec",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if not args.spec.exists():
        sys.exit(f"spec not found: {args.spec}")
    spec = json.loads(args.spec.read_text(encoding="utf-8"))
    variants = collect_variants(spec)

    args.out_rs.parent.mkdir(parents=True, exist_ok=True)
    args.out_names.parent.mkdir(parents=True, exist_ok=True)
    source_label = args.source_label or str(args.spec)
    args.out_rs.write_text(emit(variants, source_label), encoding="utf-8")
    args.out_names.write_text("\n".join(v["rust_type"] for v in variants) + "\n", encoding="utf-8")

    alt_count = sum(len(v["alts"]) for v in variants)
    print(f"Wrote {args.out_rs} ({len(variants)} enums, {alt_count} alternatives)", file=sys.stderr)
    print(f"Wrote {args.out_names}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
