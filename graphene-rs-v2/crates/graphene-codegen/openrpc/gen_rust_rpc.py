#!/usr/bin/env python3
"""Generate raw Rust OpenRPC method parameter structs from an OpenRPC spec.

This is intentionally a raw RPC compatibility layer. It does not try to design
an ergonomic SDK. It maps OpenRPC method params/results to Rust types generated
from `components.schemas` by typify plus custom static-variant generation.
"""

from __future__ import annotations

import argparse
import json
import keyword
import re
import sys
from pathlib import Path
from typing import Any


RUST_KEYWORDS = {
    "as", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
    "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
    "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
}

INT_FORMAT_MAP = {
    "int8": "i8",
    "int16": "i16",
    "int32": "i32",
    "int64": "i64",
    "uint8": "u8",
    "uint16": "u16",
    "uint32": "u32",
    "uint64": "u64",
}


def snake_to_pascal(name: str) -> str:
    return "".join(part[:1].upper() + part[1:] for part in name.split("_") if part)


def method_struct_name(method_name: str) -> str:
    return f"{snake_to_pascal(method_name)}Params"


def rust_field_name(name: str) -> str:
    clean = re.sub(r"[^A-Za-z0-9_]", "_", name)
    if not clean or clean[0].isdigit():
        clean = f"param_{clean}"
    if clean in RUST_KEYWORDS or keyword.iskeyword(clean):
        return f"r#{clean}"
    return clean


def rust_doc(text: str, indent: str = "") -> list[str]:
    if not text:
        return []
    lines = []
    for line in text.strip().splitlines():
        line = line.strip()
        if line:
            lines.append(f"{indent}/// {line}")
        else:
            lines.append(f"{indent}///")
    return lines


def ref_name(ref: str) -> str:
    return ref.rsplit("/", 1)[-1]


def schema_rust_type(schema: dict[str, Any] | None) -> str:
    if not schema:
        return "()"

    if "$ref" in schema:
        target = ref_name(schema["$ref"])
        if target == "GrapheneTimePointSec":
            return "::graphene_rpc::GrapheneTimePointSec"
        if target == "GrapheneInt64":
            return "::graphene_rpc::GrapheneInt64"
        if target == "GrapheneUInt64":
            return "::graphene_rpc::GrapheneUInt64"
        return snake_to_pascal(target)

    if "oneOf" in schema and isinstance(schema["oneOf"], list):
        variants = schema["oneOf"]
        non_null = [item for item in variants if item.get("type") != "null"]
        if len(non_null) == 1 and len(non_null) != len(variants):
            return f"Option<{schema_rust_type(non_null[0])}>"
        return "serde_json::Value"

    ty = schema.get("type")
    if isinstance(ty, list):
        non_null = [item for item in ty if item != "null"]
        if len(non_null) == 1 and len(non_null) != len(ty):
            return f"Option<{schema_rust_type({**schema, 'type': non_null[0]})}>"
        return "serde_json::Value"

    if ty == "null":
        return "()"
    if ty == "boolean":
        return "bool"
    if ty == "string":
        if schema.get("format") == "date-time":
            return "::graphene_rpc::GrapheneTimePointSec"
        return "String"
    if ty == "integer":
        return INT_FORMAT_MAP.get(schema.get("format", ""), "i64")
    if ty == "number":
        return "f32" if schema.get("format") == "float" else "f64"
    if ty == "array":
        prefix_items = schema.get("prefixItems")
        if isinstance(prefix_items, list):
            return f"({', '.join(schema_rust_type(item) for item in prefix_items)})"
        items = schema.get("items", {})
        if items == {}:
            return "Vec<serde_json::Value>"
        return f"Vec<{schema_rust_type(items)}>"
    if ty == "object":
        additional = schema.get("additionalProperties")
        if isinstance(additional, dict):
            return f"std::collections::BTreeMap<String, {schema_rust_type(additional)}>"
        return "std::collections::BTreeMap<String, serde_json::Value>"
    return "serde_json::Value"


def method_result_type(method: dict[str, Any]) -> str:
    if method.get("name") == "get_required_fees":
        return "Vec<RequiredFee>"
    if method.get("name") == "lookup_vote_ids":
        return "Vec<LookupVoteIdObject>"
    result = method.get("result") or {}
    return schema_rust_type(result.get("schema"))


def emit_required_fee() -> list[str]:
    return [
        "/// Typed fee result returned by `get_required_fees`.",
        "///",
        "/// Normal operations return a single `asset` fee. Proposal-create operations",
        "/// return an FC pair of the proposal fee and recursively nested proposed",
        "/// operation fees: `[fee, [nested_fee, ...]]`.",
        "#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]",
        "#[serde(untagged)]",
        "pub enum RequiredFee {",
        "    Asset(Asset),",
        "    ProposalCreate((Asset, Vec<RequiredFee>)),",
        "}",
        "",
    ]


def emit_lookup_vote_id_object() -> list[str]:
    return [
        "/// Typed object union returned by `lookup_vote_ids`.",
        "///",
        "/// Graphene returns concrete vote target objects in one heterogeneous array.",
        "/// The object `id` space identifies the concrete shape: committee members",
        "/// use `1.5.x`, witnesses use `1.6.x`, and workers use the worker object space.",
        "#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]",
        "#[serde(untagged)]",
        "pub enum LookupVoteIdObject {",
        "    CommitteeMember(CommitteeMemberObject),",
        "    Witness(WitnessObject),",
        "    Worker(WorkerObject),",
        "}",
        "",
    ]


def emit(spec: dict[str, Any], source_label: str) -> str:
    methods = spec.get("methods")
    if not isinstance(methods, list):
        raise ValueError("OpenRPC spec is missing methods array")

    out: list[str] = [
        "// AUTO-GENERATED by graphene-rs-v2/crates/graphene-codegen/openrpc/gen_rust_rpc.py; do not edit by hand.",
        f"// Source: {source_label} (methods)",
        "",
        "use graphene_rpc::OpenRpcParams;",
        "",
    ]

    method_names = [
        method.get("name")
        for method in methods
        if isinstance(method.get("name"), str) and method.get("name")
    ]
    out.append("/// OpenRPC method names generated for this chain crate.")
    out.append("pub const OPENRPC_METHODS: &[&str] = &[")
    for name in method_names:
        out.append(f"    {json.dumps(name)},")
    out.append("];")
    out.append("")

    if "get_required_fees" in method_names:
        out.extend(emit_required_fee())
    if "lookup_vote_ids" in method_names:
        out.extend(emit_lookup_vote_id_object())

    used_structs: set[str] = set()
    for method in methods:
        name = method.get("name")
        if not isinstance(name, str) or not name:
            continue
        struct_name = method_struct_name(name)
        if struct_name in used_structs:
            raise ValueError(f"duplicate generated RPC params struct: {struct_name}")
        used_structs.add(struct_name)
        params = method.get("params") or []
        if not isinstance(params, list):
            params = []
        result_type = method_result_type(method)

        out.extend(rust_doc(method.get("description", "")))
        if params:
            out.append("#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]")
            out.append(f"pub struct {struct_name} {{")
            for param in params:
                param_name = param.get("name")
                if not isinstance(param_name, str) or not param_name:
                    raise ValueError(f"method {name} has unnamed param")
                field_name = rust_field_name(param_name)
                out.extend(rust_doc(param.get("description", ""), "    "))
                out.append(f"    pub {field_name}: {schema_rust_type(param.get('schema'))},")
            out.append("}")
        else:
            out.append("#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]")
            out.append(f"pub struct {struct_name};")
        out.append("")

        out.append(f"impl OpenRpcParams for {struct_name} {{")
        out.append(f"    const METHOD: &'static str = {json.dumps(name)};")
        out.append(f"    type Response = {result_type};")
        out.append("")
        out.append("    fn into_positional_params(self) -> Vec<serde_json::Value> {")
        if params:
            out.append("        vec![")
            for param in params:
                field_name = rust_field_name(param["name"])
                out.append(f"            serde_json::json!(self.{field_name}),")
            out.append("        ]")
        else:
            out.append("        Vec::new()")
        out.append("    }")
        out.append("}")
        out.append("")

    return "\n".join(out)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", required=True, type=Path, help="OpenRPC spec JSON path")
    parser.add_argument("--out-rs", required=True, type=Path, help="Rust file to write")
    parser.add_argument("--source-label", default=None, help="label for generated header")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if not args.spec.exists():
        sys.exit(f"spec not found: {args.spec}")
    spec = json.loads(args.spec.read_text(encoding="utf-8"))
    args.out_rs.parent.mkdir(parents=True, exist_ok=True)
    source_label = args.source_label or str(args.spec)
    source = emit(spec, source_label)
    args.out_rs.write_text(source, encoding="utf-8")
    print(f"Wrote {args.out_rs} ({len(spec.get('methods', []))} methods)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
