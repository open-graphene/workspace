#!/usr/bin/env python3
"""Convert Doxygen XML for a Graphene FC_API class into an OpenRPC document.

Invoked by gen_wallet_openrpc.sh — not meant to be run standalone in normal use.

Strategy
--------
1. Read the FC_API(qualified::api_class, (m1)(m2)...) list directly from the API
   header. Only methods registered there are exposed over JSON-RPC, so this is
   our authoritative method whitelist.
2. Find the Doxygen XML file for the requested API class (file name is
   hash-mangled by Doxygen, so we look it up via the index).
3. For each <memberdef kind="function" prot="public"> whose name appears in the
   FC_API list, extract: return type, parameter list, brief & detailed docs.
4. Map each C++ type string to a JSON Schema fragment. Complex types become
   $ref pointers into components.schemas — those schemas are expected to be
   filled in by a separate generator that walks the FC_REFLECT graph
   (see programs/js_operation_serializer).
5. Emit an OpenRPC 1.3.2 document on stdout.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from pathlib import Path


# ---------------------------------------------------------------------------
# FC_API list extraction
# ---------------------------------------------------------------------------

METHOD_TOKEN_RE = re.compile(r"\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)")


def parse_fc_api_methods(api_header: Path, api_qualified_name: str) -> set[str]:
    """Locate FC_API(api_qualified_name, (m1)(m2)...) and pull out the method
    tokens. We can't use a single regex because the body contains nested `(...)`
    groups — instead we find the macro start, then walk forward counting parens
    until the outer call closes."""
    text = api_header.read_text(encoding="utf-8")
    escaped_name = re.escape(api_qualified_name).replace(r"\::", r"\s*::\s*")
    start_re = re.compile(rf"FC_API\s*\(\s*{escaped_name}\s*,")
    m = start_re.search(text)
    if not m:
        sys.exit(f"FC_API({api_qualified_name}, ...) not found in {api_header}")
    # Depth is 1 right after the '(' that opens the macro call.
    depth = 1
    i = m.end()
    body_start = i
    while i < len(text) and depth > 0:
        ch = text[i]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                break
        i += 1
    if depth != 0:
        sys.exit(f"Unterminated FC_API macro in {wallet_header}")
    body = text[body_start:i]
    return set(METHOD_TOKEN_RE.findall(body))


# ---------------------------------------------------------------------------
# Doxygen XML helpers
# ---------------------------------------------------------------------------

def find_api_xml(xml_dir: Path, api_qualified_name: str) -> Path:
    index = ET.parse(xml_dir / "index.xml").getroot()
    for compound in index.findall("./compound[@kind='class']"):
        name_el = compound.find("name")
        if name_el is not None and name_el.text == api_qualified_name:
            return xml_dir / f"{compound.get('refid')}.xml"
    sys.exit(f"{api_qualified_name} not found in Doxygen index.xml")


def text_of(el: ET.Element | None) -> str:
    """Flatten an element's text content (ignoring formatting tags)."""
    if el is None:
        return ""
    parts: list[str] = []
    if el.text:
        parts.append(el.text)
    for child in el:
        parts.append(text_of(child))
        if child.tail:
            parts.append(child.tail)
    return "".join(parts)


def collapse_ws(s: str) -> str:
    return re.sub(r"\s+", " ", s).strip()


def extract_description(member: ET.Element) -> tuple[str, str]:
    """Return (summary, description) — brief and detailed Doxygen blocks."""
    brief = collapse_ws(text_of(member.find("briefdescription")))
    detailed_el = member.find("detaileddescription")
    if detailed_el is None:
        return brief, ""

    paragraphs: list[str] = []
    for para in detailed_el.findall("para"):
        # Skip the parameter / return blocks — they are extracted separately
        if para.find("parameterlist") is not None or para.find("simplesect") is not None:
            inline = "".join(
                text_of(c) if c.tag not in {"parameterlist", "simplesect"} else ""
                for c in para.iter()
                if c is para or c.getparent() is para  # type: ignore[attr-defined]
            ) if False else ""  # ElementTree has no getparent; fall through to safe path
            # Safe fallback: collect only direct text/tails, ignoring nested blocks
            buf: list[str] = []
            if para.text:
                buf.append(para.text)
            for child in para:
                if child.tag in {"parameterlist", "simplesect"}:
                    if child.tail:
                        buf.append(child.tail)
                    continue
                buf.append(text_of(child))
                if child.tail:
                    buf.append(child.tail)
            inline = "".join(buf)
            inline = collapse_ws(inline)
            if inline:
                paragraphs.append(inline)
        else:
            txt = collapse_ws(text_of(para))
            if txt:
                paragraphs.append(txt)
    return brief, "\n\n".join(paragraphs)


def extract_param_docs(member: ET.Element) -> tuple[dict[str, str], str]:
    """Return ({param_name: doc}, return_doc) from Doxygen @param and @return."""
    param_docs: dict[str, str] = {}
    return_doc = ""
    for plist in member.findall(".//parameterlist[@kind='param']"):
        for item in plist.findall("parameteritem"):
            names = [collapse_ws(text_of(n)) for n in item.findall(".//parametername")]
            description = collapse_ws(text_of(item.find("parameterdescription")))
            for n in names:
                if n:
                    param_docs[n] = description
    for sect in member.findall(".//simplesect[@kind='return']"):
        return_doc = collapse_ws(text_of(sect))
    return param_docs, return_doc


# ---------------------------------------------------------------------------
# C++ type → JSON Schema mapping
# ---------------------------------------------------------------------------

PRIMITIVE_MAP: dict[str, dict] = {
    "void":     {"type": "null"},
    "bool":     {"type": "boolean"},
    "string":   {"type": "string"},
    "char":     {"type": "string", "minLength": 1, "maxLength": 1},
    "int8_t":   {"type": "integer", "format": "int8"},
    "int16_t":  {"type": "integer", "format": "int16"},
    "int32_t":  {"type": "integer", "format": "int32"},
    "int64_t":  {"type": "integer", "format": "int64"},
    "uint8_t":  {"type": "integer", "format": "uint8",  "minimum": 0},
    "uint16_t": {"type": "integer", "format": "uint16", "minimum": 0},
    "uint32_t": {"type": "integer", "format": "uint32", "minimum": 0},
    "uint64_t": {"$ref": "#/components/schemas/GrapheneUInt64"},
    "float":    {"type": "number", "format": "float"},
    "double":   {"type": "number", "format": "double"},
    "share_type": {"type": "integer", "format": "int64"},
    "time_point_sec": {"$ref": "#/components/schemas/GrapheneTimePointSec"},
    "fc::time_point_sec": {"$ref": "#/components/schemas/GrapheneTimePointSec"},
    "time_point": {"$ref": "#/components/schemas/GrapheneTimePointSec"},
    "fc::time_point": {"$ref": "#/components/schemas/GrapheneTimePointSec"},
    "object_id_type": {"type": "string", "pattern": r"^\d+\.\d+\.\d+$"},
    "public_key_type": {"type": "string"},
    "private_key_type": {"type": "string"},
    "signature_type": {"type": "string"},
    "fc::ecc::compact_signature": {"type": "string"},
    "ripemd160": {"type": "string"},
    "sha256": {"type": "string"},
    "fc::ripemd160": {"type": "string"},
    "fc::sha256": {"type": "string"},
    "variant": {},  # accept anything
    "fc::variant": {},
    "variants": {"type": "array", "items": {}},
    "fc::variants": {"type": "array", "items": {}},
    "variant_object": {"type": "object"},
    "fc::variant_object": {"type": "object"},
    "mutable_variant_object": {"type": "object"},
    "fc::mutable_variant_object": {"type": "object"},
    "blind_factor_type": {"type": "string"},
    "commitment_type": {"type": "string"},
}

# graphene::protocol object id helpers — they all serialize as "1.X.Y" strings
ID_TYPES = {
    "account_id_type", "asset_id_type", "force_settlement_id_type",
    "committee_member_id_type", "witness_id_type", "limit_order_id_type",
    "call_order_id_type", "custom_id_type", "proposal_id_type",
    "operation_history_id_type", "withdraw_permission_id_type",
    "vesting_balance_id_type", "worker_id_type", "balance_id_type",
    "htlc_id_type", "custom_authority_id_type", "ticket_id_type",
    "liquidity_pool_id_type", "samet_fund_id_type", "credit_offer_id_type",
    "credit_deal_id_type",
}
for _t in ID_TYPES:
    PRIMITIVE_MAP.setdefault(_t, {"type": "string", "pattern": r"^\d+\.\d+\.\d+$"})


def clean_type(raw: str) -> str:
    """Strip Doxygen noise from a C++ type expression."""
    s = raw
    s = re.sub(r"\bconst\b", "", s)
    s = s.replace("&", " ")
    s = re.sub(r",\s*std::\s*less<>", "", s)
    s = re.sub(r",\s*std::less<\s*\w+\s*>", "", s)
    s = s.replace("std::", "")
    s = s.replace("fc::", "fc::")  # keep fc:: so we can match in PRIMITIVE_MAP
    s = re.sub(r"\s+", " ", s).strip()
    s = s.replace("< ", "<").replace(" >", ">")
    return s


# Container patterns: (regex, schema_builder taking inner schemas)
def _array(inner: dict) -> dict:           return {"type": "array", "items": inner}
def _set_array(inner: dict) -> dict:       return {"type": "array", "items": inner, "uniqueItems": True}
def _map_object(_k: dict, v: dict) -> dict: return {"type": "object", "additionalProperties": v}
def _nullable(inner: dict) -> dict:
    if "$ref" in inner:
        return {"oneOf": [inner, {"type": "null"}]}
    out = dict(inner)
    existing = out.get("type")
    if isinstance(existing, str):
        out["type"] = [existing, "null"]
    elif isinstance(existing, list) and "null" not in existing:
        out["type"] = existing + ["null"]
    else:
        return {"oneOf": [inner, {"type": "null"}]}
    return out


SINGLE_GENERIC = [
    ("vector",   _array),
    ("deque",    _array),
    ("list",     _array),
    ("set",      _set_array),
    ("flat_set", _set_array),
    ("optional", _nullable),
    ("fc::optional", _nullable),
    ("safe",     lambda inner: inner),
]


def split_template_args(args: str) -> list[str]:
    """Split a comma-separated C++ template argument list, respecting nesting."""
    depth = 0
    out: list[str] = []
    buf: list[str] = []
    for ch in args:
        if ch == "<":
            depth += 1
            buf.append(ch)
        elif ch == ">":
            depth -= 1
            buf.append(ch)
        elif ch == "," and depth == 0:
            out.append("".join(buf).strip())
            buf = []
        else:
            buf.append(ch)
    if buf:
        out.append("".join(buf).strip())
    return out


def map_type(cpp_type: str, schemas: dict[str, dict]) -> dict:
    """Translate a cleaned C++ type expression into a JSON Schema fragment.

    Unknown leaf types are recorded as $ref placeholders in ``schemas`` so a
    downstream pass (e.g. operations-serializer JSON Schema dump) can fill them.
    """
    t = cpp_type.strip()
    if not t:
        return {}

    # Primitive / known leaf
    if t in PRIMITIVE_MAP:
        return dict(PRIMITIVE_MAP[t])

    # Container?
    m = re.match(r"^([A-Za-z_][A-Za-z0-9_:]*)\s*<(.+)>\s*$", t)
    if m:
        outer = m.group(1)
        inner_args = split_template_args(m.group(2))

        for name, builder in SINGLE_GENERIC:
            if outer == name and len(inner_args) == 1:
                return builder(map_type(inner_args[0], schemas))

        if outer in {"map", "flat_map", "unordered_map"} and len(inner_args) >= 2:
            return _map_object(map_type(inner_args[0], schemas), map_type(inner_args[1], schemas))

        if outer == "pair" and len(inner_args) == 2:
            return {
                "type": "array",
                "prefixItems": [map_type(a, schemas) for a in inner_args],
                "minItems": 2,
                "maxItems": 2,
            }

        if outer == "tuple":
            return {
                "type": "array",
                "prefixItems": [map_type(a, schemas) for a in inner_args],
                "minItems": len(inner_args),
                "maxItems": len(inner_args),
            }

        # Unknown generic — record a placeholder for the whole expression.
        leaf = re.sub(r"[^A-Za-z0-9_]", "_", t).strip("_")
        schemas.setdefault(leaf, {"x-cpp-type": t, "description": "TODO: fill from FC_REFLECT walker"})
        return {"$ref": f"#/components/schemas/{leaf}"}

    # Bare leaf — register a placeholder schema and emit a $ref.
    # Use the same naming convention as gen_types_spec.py (`::` -> `__`) so
    # both passes end up referencing identical schema names. A mismatch
    # caused duplicate Rust types when SDK generators (typify) CamelCased
    # `foo_bar_extensions_type` and `foo_bar__extensions_type` into the same
    # identifier.
    leaf = re.sub(r"[^A-Za-z0-9_]", "_", t).strip("_")
    schemas.setdefault(leaf, {"x-cpp-type": t, "description": "TODO: fill from FC_REFLECT walker"})
    return {"$ref": f"#/components/schemas/{leaf}"}


# ---------------------------------------------------------------------------
# Method extraction
# ---------------------------------------------------------------------------

@dataclass
class Param:
    name: str
    cpp_type: str
    description: str = ""


@dataclass
class Method:
    name: str
    return_cpp_type: str
    params: list[Param] = field(default_factory=list)
    summary: str = ""
    description: str = ""
    return_doc: str = ""


def parse_methods(class_xml: Path, allowed: set[str]) -> list[Method]:
    root = ET.parse(class_xml).getroot()
    out: list[Method] = []
    seen: set[str] = set()
    for member in root.findall(".//memberdef[@kind='function']"):
        if member.get("prot") != "public":
            continue
        name_el = member.find("name")
        if name_el is None or name_el.text is None:
            continue
        name = name_el.text.strip()
        if name not in allowed or name in seen:
            continue
        seen.add(name)

        return_type = clean_type(text_of(member.find("type")))
        param_docs, return_doc = extract_param_docs(member)
        summary, description = extract_description(member)

        params: list[Param] = []
        for p in member.findall("param"):
            ptype = clean_type(text_of(p.find("type")))
            decl_el = p.find("declname")
            pname = (decl_el.text or "").strip() if decl_el is not None and decl_el.text else ""
            if not pname:
                # Fall back to <defname> or positional name
                defname_el = p.find("defname")
                pname = (defname_el.text or "").strip() if defname_el is not None and defname_el.text else f"arg{len(params)}"
            params.append(Param(name=pname, cpp_type=ptype, description=param_docs.get(pname, "")))

        out.append(Method(
            name=name,
            return_cpp_type=return_type,
            params=params,
            summary=summary,
            description=description,
            return_doc=return_doc,
        ))

    out.sort(key=lambda m: m.name)
    return out


# ---------------------------------------------------------------------------
# OpenRPC document assembly
# ---------------------------------------------------------------------------

def build_openrpc(methods: list[Method], *, api_qualified_name: str, title: str) -> dict:
    schemas: dict[str, dict] = {}
    openrpc_methods: list[dict] = []

    for m in methods:
        params_doc: list[dict] = []
        for p in m.params:
            cd: dict = {
                "name": p.name,
                "schema": map_type(p.cpp_type, schemas),
                "required": True,
            }
            if p.description:
                cd["description"] = p.description
            if p.cpp_type:
                cd["x-cpp-type"] = p.cpp_type
            params_doc.append(cd)

        result: dict = {
            "name": f"{m.name}_result",
            "schema": map_type(m.return_cpp_type, schemas),
        }
        if m.return_doc:
            result["description"] = m.return_doc
        if m.return_cpp_type:
            result["x-cpp-type"] = m.return_cpp_type

        entry: dict = {"name": m.name, "params": params_doc, "result": result}
        if m.summary:
            entry["summary"] = m.summary
        if m.description:
            entry["description"] = m.description
        openrpc_methods.append(entry)

    return {
        "openrpc": "1.3.2",
        "info": {
            "title": title,
            "version": "0.1.0",
            "description": (
                f"Generated from {api_qualified_name} via Doxygen XML.\n"
                "Schemas marked with x-cpp-type are placeholders awaiting fill-in "
                "from an FC_REFLECT walker."
            ),
        },
        "methods": openrpc_methods,
        "components": {"schemas": schemas},
    }


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--xml-dir", required=True, type=Path)
    ap.add_argument("--api-header", type=Path)
    ap.add_argument("--api-qualified-name", default="graphene::wallet::wallet_api")
    ap.add_argument("--title", default=None)
    ap.add_argument("--exclude-method", action="append", default=[])
    # Backward-compatible alias while the pipeline moves from wallet-only to API-class-aware.
    ap.add_argument("--wallet-header", type=Path)
    args = ap.parse_args()

    api_header = args.api_header or args.wallet_header
    if api_header is None:
        sys.exit("missing --api-header")

    excluded = set(args.exclude_method)
    allowed = parse_fc_api_methods(api_header, args.api_qualified_name) - excluded
    class_xml = find_api_xml(args.xml_dir, args.api_qualified_name)
    methods = parse_methods(class_xml, allowed)

    missing = allowed - {m.name for m in methods}
    if missing:
        print(
            f"warning: {len(missing)} FC_API methods missing from Doxygen output: "
            f"{', '.join(sorted(missing))}",
            file=sys.stderr,
        )

    doc = build_openrpc(
        methods,
        api_qualified_name=args.api_qualified_name,
        title=args.title or args.api_qualified_name,
    )
    json.dump(doc, sys.stdout, indent=2, ensure_ascii=False)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
