#!/usr/bin/env python3
"""bin/gen_types_spec.py — fill the placeholder schemas left by gen_wallet_spec.py.

Reads an existing OpenRPC document (`--spec`) and a set of C++ headers
(`--header-root` — passed once per root, walked recursively), then:

1. Extracts every FC_REFLECT / FC_REFLECT_DERIVED / FC_REFLECT_ENUM /
   FC_REFLECT_TYPENAME / FC_REFLECT_EMPTY macro to learn which fields each type
   exposes over the wire and how they relate (e.g. inheritance for DERIVED).
2. Walks every `struct` / `class` declaration so we know the C++ type of each
   reflected field. Scope (`namespace foo { class Bar { struct Baz { ... } } }`)
   is tracked so nested names like `bitasset_options::ext` resolve correctly.
3. Records `using NAME = TYPE;` and `typedef TYPE NAME;` aliases, plus the
   `fc::static_variant<...>` alternatives used by the discriminated unions
   (`operation`, `op_result`, ...).
4. Walks the placeholders in `components.schemas`, replaces them with concrete
   JSON Schema objects, transitively pulls in newly referenced types, and
   writes the updated document back to `--spec` (or `--output`).

This is the FC reflection pass that the wallet generator deferred — its docs
are written in the form: when the wallet generator emits a $ref to type X, this
script's job is to make sure the matching components.schemas entry actually
describes X.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path


# ---------------------------------------------------------------------------
# Preprocessing: strip comments
# ---------------------------------------------------------------------------

_BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.DOTALL)
_LINE_COMMENT_RE = re.compile(r"//[^\n]*")


def strip_comments(s: str) -> str:
    """Remove C/C++ comments while preserving line counts (helps debugging)."""

    def repl_block(m: re.Match[str]) -> str:
        return "".join("\n" if c == "\n" else " " for c in m.group(0))

    s = _BLOCK_COMMENT_RE.sub(repl_block, s)
    s = _LINE_COMMENT_RE.sub("", s)
    return s


# Matches an object-like `#define NAME body` (no argument list).
# Handles backslash-newline continuations greedily until a non-continued newline.
_DEFINE_RE = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+(\w+)(?![\w(])[ \t]+((?:.*?\\\n)*.*)",
    re.MULTILINE,
)


def preprocess(src: str) -> str:
    """Strip comments, then expand object-like ``#define`` macros within the
    same translation unit.

    We need this because some variant alternative lists are macros — e.g.
    ``static_variant<GRAPHENE_OP_RESTRICTION_ARGUMENTS_VARIADIC>`` in
    ``restriction.hpp`` expands to 42 comma-separated types. Without expansion
    we'd treat the macro name as a single alternative.

    Function-like macros (``#define FOO(x) ...``) are deliberately skipped —
    parsing those correctly needs more than text substitution and isn't
    required for FC reflection patterns we care about.
    """
    src = strip_comments(src)
    macros: dict[str, str] = {}
    for m in _DEFINE_RE.finditer(src):
        name = m.group(1)
        body = m.group(2)
        # Join backslash-newline continuations into a single line.
        body = re.sub(r"\\\s*\n", " ", body)
        body = re.sub(r"\s+", " ", body).strip()
        if not body:
            continue
        macros[name] = body
    # Single-pass replacement of each defined macro's bare name. We don't bother
    # with iterative expansion — none of the cases that matter chain.
    for name, body in macros.items():
        src = re.sub(r"\b" + re.escape(name) + r"\b", body, src)
    return src


# ---------------------------------------------------------------------------
# Brace-matching utility
# ---------------------------------------------------------------------------

def matching_brace(s: str, open_idx: int) -> int:
    """Given s[open_idx] == '{', return index of matching '}'. -1 if not found."""
    assert s[open_idx] == "{"
    depth = 0
    i = open_idx
    while i < len(s):
        c = s[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def matching_paren(s: str, open_idx: int) -> int:
    assert s[open_idx] == "("
    depth = 0
    i = open_idx
    while i < len(s):
        c = s[i]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


# ---------------------------------------------------------------------------
# C++ identifier scanner: extract structs / namespaces / using / variants
# ---------------------------------------------------------------------------

@dataclass
class StructInfo:
    qualified_name: str
    body: str
    bases: list[str] = field(default_factory=list)
    source_file: str = ""


@dataclass
class EnumInfo:
    qualified_name: str
    values: list[str] = field(default_factory=list)
    is_scoped: bool = False


@dataclass
class AliasInfo:
    qualified_name: str
    cpp_type: str


@dataclass
class VariantInfo:
    qualified_name: str
    alternatives: list[str]


# Matches `namespace NAME {` (skip `namespace { ... }` anonymous)
_NAMESPACE_RE = re.compile(r"\bnamespace\s+([A-Za-z_]\w*)\s*\{")
# Matches `struct|class NAME [final] [: BASES] {` — body parsed via brace matching
_STRUCT_RE = re.compile(
    r"\b(?P<kw>struct|class)\s+(?P<name>[A-Za-z_]\w*)\b"
    r"(?:\s+final)?"
    r"\s*(?::\s*(?P<bases>[^{};]+))?\s*\{"
)
# `enum [class|struct] NAME [: TYPE] { ... };`
_ENUM_RE = re.compile(
    r"\benum\s+(?P<scoped>class|struct)?\s*(?P<name>[A-Za-z_]\w*)\s*"
    r"(?::\s*[A-Za-z_:\w]+\s*)?\{"
)
# `using NAME = TYPE;`
_USING_RE = re.compile(r"\busing\s+([A-Za-z_]\w*)\s*=\s*([^;]+);")
# `typedef TYPE NAME;`
_TYPEDEF_RE = re.compile(r"\btypedef\s+([^;]+?)\s+([A-Za-z_]\w*)\s*;")


def parse_cpp(src: str, source_file: str, *, structs: dict[str, StructInfo],
              enums: dict[str, EnumInfo], aliases: dict[str, AliasInfo],
              variants: dict[str, VariantInfo]) -> None:
    """Walk src (comments stripped, macros expanded) and populate the registries.

    Scope tracking is explicit: namespaces and class bodies both push onto
    ``scope`` so qualified names like ``graphene::protocol::bitasset_options::ext``
    are assembled correctly.
    """
    _walk(src, scope=[], structs=structs, enums=enums, aliases=aliases,
          variants=variants, source_file=source_file)


def _qualify(scope: list[str], name: str) -> str:
    return "::".join(scope + [name]) if scope else name


def _walk(region: str, *, scope: list[str], structs, enums, aliases,
          variants, source_file: str) -> None:
    """Scan ``region`` for declarations. Recurses into namespace/struct bodies."""
    i = 0
    n = len(region)
    while i < n:
        # Skip strings (rare in headers but exist)
        c = region[i]
        if c == '"':
            j = i + 1
            while j < n and region[j] != '"':
                if region[j] == "\\" and j + 1 < n:
                    j += 2
                    continue
                j += 1
            i = j + 1
            continue

        # namespace
        m = _NAMESPACE_RE.match(region, i)
        if m:
            ns_name = m.group(1)
            brace = region.find("{", m.end() - 1)
            if brace == -1:
                i = m.end()
                continue
            close = matching_brace(region, brace)
            if close == -1:
                i = m.end()
                continue
            body = region[brace + 1:close]
            _walk(body, scope=scope + [ns_name], structs=structs, enums=enums,
                  aliases=aliases, variants=variants, source_file=source_file)
            i = close + 1
            continue

        # struct / class
        m = _STRUCT_RE.match(region, i)
        if m:
            name = m.group("name")
            bases_raw = m.group("bases") or ""
            brace = m.end() - 1  # the '{'
            close = matching_brace(region, brace)
            if close == -1:
                i = m.end()
                continue
            body = region[brace + 1:close]

            # Check this isn't a variable declaration (`struct S s;`) — these
            # don't have a `{` immediately after the name, but our regex
            # requires `{`, so we're already safe.
            qname = _qualify(scope, name)
            structs[qname] = StructInfo(
                qualified_name=qname,
                body=body,
                bases=_parse_bases(bases_raw),
                source_file=source_file,
            )
            # Recurse to capture nested structs / enums inside the body
            _walk(body, scope=scope + [name], structs=structs, enums=enums,
                  aliases=aliases, variants=variants, source_file=source_file)
            # Body may be followed by `} name;` (variable) or `};` — skip past.
            i = close + 1
            continue

        # enum [class] NAME { ... };
        m = _ENUM_RE.match(region, i)
        if m:
            name = m.group("name")
            scoped = m.group("scoped") is not None
            brace = m.end() - 1
            close = matching_brace(region, brace)
            if close == -1:
                i = m.end()
                continue
            body = region[brace + 1:close]
            values = _parse_enum_values(body)
            qname = _qualify(scope, name)
            enums[qname] = EnumInfo(qualified_name=qname, values=values, is_scoped=scoped)
            i = close + 1
            continue

        # using NAME = TYPE;
        m = _USING_RE.match(region, i)
        if m:
            name = m.group(1)
            ty = m.group(2).strip()
            qname = _qualify(scope, name)
            # Detect static_variant alternatives
            sv = _match_static_variant(ty)
            if sv is not None:
                variants[qname] = VariantInfo(qualified_name=qname, alternatives=sv)
            else:
                aliases[qname] = AliasInfo(qualified_name=qname, cpp_type=ty)
            i = m.end()
            continue

        # typedef TYPE NAME;
        m = _TYPEDEF_RE.match(region, i)
        if m:
            ty = m.group(1).strip()
            name = m.group(2)
            qname = _qualify(scope, name)
            sv = _match_static_variant(ty)
            if sv is not None:
                variants[qname] = VariantInfo(qualified_name=qname, alternatives=sv)
            else:
                aliases[qname] = AliasInfo(qualified_name=qname, cpp_type=ty)
            i = m.end()
            continue

        i += 1


def _parse_bases(raw: str) -> list[str]:
    if not raw.strip():
        return []
    out: list[str] = []
    for part in raw.split(","):
        part = part.strip()
        # strip access specifier
        for kw in ("public ", "private ", "protected ", "virtual "):
            if part.startswith(kw):
                part = part[len(kw):]
        out.append(part.strip())
    return out


def _parse_enum_values(body: str) -> list[str]:
    out: list[str] = []
    for raw in body.split(","):
        name_part = raw.split("=")[0].strip()
        if name_part:
            out.append(name_part)
    return out


_STATIC_VARIANT_RE = re.compile(
    r"^(?:fc::)?static_variant\s*<(?P<args>.*)>\s*$", re.DOTALL
)


def _match_static_variant(ty: str) -> list[str] | None:
    """Detect ``fc::static_variant<A, B, ...>`` (with or without ``fc::``)."""
    m = _STATIC_VARIANT_RE.match(ty.strip())
    if not m:
        return None
    return _split_template_args(m.group("args"))


def _split_template_args(args: str) -> list[str]:
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
        tail = "".join(buf).strip()
        if tail:
            out.append(tail)
    return out


# ---------------------------------------------------------------------------
# FC_REFLECT extraction
# ---------------------------------------------------------------------------

@dataclass
class ReflectedStruct:
    qualified_name: str
    base_names: list[str]  # for FC_REFLECT_DERIVED
    field_names: list[str]


@dataclass
class ReflectedEnum:
    qualified_name: str
    values: list[str]


@dataclass
class ReflectedTypename:
    qualified_name: str


_REFLECT_MACROS = {
    "FC_REFLECT",
    "FC_REFLECT_DERIVED",
    "FC_REFLECT_DERIVED_NO_TYPENAME",
    "FC_REFLECT_NO_TYPENAME",
    "FC_REFLECT_ENUM",
    "FC_REFLECT_TYPENAME",
    "FC_REFLECT_EMPTY",
}
# Order matters: longest macros first so the alternation doesn't preempt them
# with a shorter prefix match.
_MACRO_RE = re.compile(
    r"\b(?P<macro>"
    r"FC_REFLECT_DERIVED_NO_TYPENAME"
    r"|FC_REFLECT_NO_TYPENAME"
    r"|FC_REFLECT_DERIVED"
    r"|FC_REFLECT_TYPENAME"
    r"|FC_REFLECT_ENUM"
    r"|FC_REFLECT_EMPTY"
    r"|FC_REFLECT"
    r")\s*\("
)
_PAREN_GROUP_RE = re.compile(r"\(([^()]*)\)")


def parse_reflect_macros(src: str) -> tuple[
    list[ReflectedStruct], list[ReflectedEnum], list[ReflectedTypename]
]:
    """Caller is expected to have already comment-stripped & macro-expanded src."""
    structs: list[ReflectedStruct] = []
    enums: list[ReflectedEnum] = []
    typenames: list[ReflectedTypename] = []

    i = 0
    while i < len(src):
        m = _MACRO_RE.search(src, i)
        if not m:
            break
        macro = m.group("macro")
        open_paren = m.end() - 1
        close = matching_paren(src, open_paren)
        if close == -1:
            break
        body = src[open_paren + 1:close]
        i = close + 1

        # Split at top-level commas to get macro args
        args = _split_template_args(body)
        if not args:
            continue
        qname = args[0].strip()

        if macro == "FC_REFLECT_EMPTY":
            # An empty FC_REFLECT struct is serialized as `{}` — model it as
            # an object with no properties.
            structs.append(ReflectedStruct(
                qualified_name=qname, base_names=[], field_names=[]))
            continue

        if macro == "FC_REFLECT_TYPENAME":
            typenames.append(ReflectedTypename(qualified_name=qname))
            continue

        if macro == "FC_REFLECT_ENUM":
            values_blob = " ".join(args[1:])
            values = _PAREN_GROUP_RE.findall(values_blob)
            values = [v.strip() for v in values if v.strip()]
            enums.append(ReflectedEnum(qualified_name=qname, values=values))
            continue

        if macro in ("FC_REFLECT_DERIVED", "FC_REFLECT_DERIVED_NO_TYPENAME"):
            bases_blob = args[1] if len(args) > 1 else ""
            fields_blob = " ".join(args[2:])
            # `BOOST_PP_SEQ_NIL` is graphene's sentinel for "no bases"
            if "BOOST_PP_SEQ_NIL" in bases_blob:
                base_names: list[str] = []
            else:
                base_names = [b.strip() for b in _PAREN_GROUP_RE.findall(bases_blob) if b.strip()]
            field_names = [f.strip() for f in _PAREN_GROUP_RE.findall(fields_blob) if f.strip()]
            structs.append(ReflectedStruct(
                qualified_name=qname, base_names=base_names, field_names=field_names))
            continue

        # FC_REFLECT / FC_REFLECT_NO_TYPENAME (name, (f1)(f2)...)
        fields_blob = " ".join(args[1:])
        field_names = [f.strip() for f in _PAREN_GROUP_RE.findall(fields_blob) if f.strip()]
        structs.append(ReflectedStruct(
            qualified_name=qname, base_names=[], field_names=field_names))

    return structs, enums, typenames


# ---------------------------------------------------------------------------
# Field-type lookup inside a struct body
# ---------------------------------------------------------------------------

_STMT_BOUNDARY = ";{}"


def find_field_type(body: str, field_name: str) -> str | None:
    """Find the C++ type declared for ``field_name`` inside ``body``.

    Strategy: replace every nested ``{...}`` block with ``{}`` so function
    bodies / nested type bodies / brace initializers don't trip us up. Then
    locate the field name followed by a terminator (``=``, ``;``, ``,``, ``(``,
    ``{``, ``}``) and walk backwards to the previous statement boundary to
    recover the type expression.
    """
    flat = _flatten_body(body)
    pattern = re.compile(
        r"(?<![\w:])" + re.escape(field_name) +
        r"\s*(?:\[[^]]*\])?\s*(?=[=;,(){}])"
    )
    for m in pattern.finditer(flat):
        # Walk backward to the previous top-level statement boundary.
        start = m.start()
        i = start - 1
        while i >= 0 and flat[i] not in _STMT_BOUNDARY:
            i -= 1
        prefix = flat[i + 1:start]
        # Strip leading access labels / qualifiers that may immediately
        # precede a field declaration.
        prefix = re.sub(r"\b(public|private|protected)\s*:\s*", "", prefix)
        prefix = re.sub(r"\b(mutable|static|inline|constexpr|thread_local)\b", "", prefix)
        prefix = prefix.strip()
        if not prefix:
            continue
        # Method declarations leave `(` in the prefix because their parens are
        # part of the signature; initializer chains leave `=`. Both disqualify.
        if "(" in prefix or "=" in prefix:
            continue
        if prefix.startswith(("typedef ", "using ", "friend ", "static_assert", "enum ", "struct ", "class ", "namespace ", "//", "/*")):
            continue
        # Multi-declaration: `Type a, b, c;` — when looking up `b` or `c`, the
        # walked-back prefix ends in `,` and includes the previous identifiers.
        # The shared type is the leading run before the first comma, minus the
        # first identifier name.
        if prefix.endswith(","):
            first_chunk = prefix.rstrip(",").split(",")[0].strip()
            m = re.match(r"^(.+?)\s+(\w+)$", first_chunk)
            if m:
                return _clean_type(m.group(1))
            continue
        last = prefix[-1]
        if last not in ">_]" and not last.isalnum() and last != ":":
            continue
        return _clean_type(prefix)
    return None


def _flatten_body(body: str) -> str:
    """Replace every `{ ... }` block (function bodies, init lists, nested
    types) with a placeholder so semicolon-splitting yields top-level statements."""
    out: list[str] = []
    i = 0
    while i < len(body):
        c = body[i]
        if c == "{":
            j = matching_brace(body, i)
            if j == -1:
                break
            # Replace block with `{}` placeholder — preserves spacing roughly
            out.append("{}")
            i = j + 1
            continue
        out.append(c)
        i += 1
    return "".join(out)


def _clean_type(raw: str) -> str:
    s = re.sub(r"\s+", " ", raw).strip()
    s = s.replace("< ", "<").replace(" >", ">")
    return s


# ---------------------------------------------------------------------------
# C++ type → JSON Schema (shares logic with gen_wallet_spec.py)
# ---------------------------------------------------------------------------

PRIMITIVE_MAP: dict[str, dict] = {
    "void":     {"type": "null"},
    "bool":     {"type": "boolean"},
    "string":   {"type": "string"},
    "std::string": {"type": "string"},
    "char":     {"type": "string", "minLength": 1, "maxLength": 1},
    "int8_t":   {"type": "integer", "format": "int8"},
    "int16_t":  {"type": "integer", "format": "int16"},
    "int32_t":  {"type": "integer", "format": "int32"},
    "int64_t":  {"type": "integer", "format": "int64"},
    "uint8_t":  {"type": "integer", "format": "uint8",  "minimum": 0},
    "uint16_t": {"type": "integer", "format": "uint16", "minimum": 0},
    "uint32_t": {"type": "integer", "format": "uint32", "minimum": 0},
    "uint64_t": {"type": "integer", "format": "uint64", "minimum": 0},
    "float":    {"type": "number", "format": "float"},
    "double":   {"type": "number", "format": "double"},
    "share_type":   {"type": "integer", "format": "int64"},
    "weight_type":  {"type": "integer", "format": "uint16", "minimum": 0},
    "time_point_sec": {"type": "string", "format": "date-time"},
    "fc::time_point_sec": {"type": "string", "format": "date-time"},
    "time_point": {"type": "string", "format": "date-time"},
    "fc::time_point": {"type": "string", "format": "date-time"},
    "object_id_type": {"type": "string", "pattern": r"^\d+\.\d+\.\d+$"},
    "public_key_type": {"type": "string"},
    "private_key_type": {"type": "string"},
    "signature_type": {"type": "string"},
    "fc::ecc::compact_signature": {"type": "string"},
    "ripemd160": {"type": "string"},
    "sha256": {"type": "string"},
    "fc::ripemd160": {"type": "string"},
    "fc::sha256": {"type": "string"},
    "variant": {},
    "fc::variant": {},
    "variant_object": {"type": "object"},
    "fc::variant_object": {"type": "object"},
    "mutable_variant_object": {"type": "object"},
    "fc::mutable_variant_object": {"type": "object"},
    "blind_factor_type": {"type": "string"},
    "commitment_type": {"type": "string"},
    "transaction_id_type": {"type": "string"},
    "uint128_t":     {"type": "string", "pattern": r"^\d+$", "description": "Unsigned 128-bit integer serialized as a decimal string."},
    "fc::uint128_t": {"type": "string", "pattern": r"^\d+$", "description": "Unsigned 128-bit integer serialized as a decimal string."},
    "int128_t":      {"type": "string", "pattern": r"^-?\d+$", "description": "Signed 128-bit integer serialized as a decimal string."},
    "fc::int128_t":  {"type": "string", "pattern": r"^-?\d+$", "description": "Signed 128-bit integer serialized as a decimal string."},
    "unsigned_int":     {"type": "integer", "minimum": 0, "description": "FC varint."},
    "fc::unsigned_int": {"type": "integer", "minimum": 0, "description": "FC varint."},
    "signed_int":       {"type": "integer", "description": "FC zig-zag varint."},
    "fc::signed_int":   {"type": "integer", "description": "FC zig-zag varint."},
    # Binary blobs typically serialized as hex strings:
    "fc::sha1":   {"type": "string", "description": "Hex-encoded SHA-1 digest."},
    "sha1":       {"type": "string", "description": "Hex-encoded SHA-1 digest."},
    "fc::hash160":{"type": "string", "description": "Hex-encoded HASH160 digest."},
    "hash160":    {"type": "string", "description": "Hex-encoded HASH160 digest."},
    "fc::ecc::commitment_type": {"type": "string", "description": "Pedersen commitment, hex-encoded."},
    "range_proof_type":         {"type": "string", "description": "Confidential range proof, hex-encoded."},
}

# Additional graphene::chain id types — same `1.X.Y` shape as the protocol ids.
_CHAIN_ID_TYPES = {
    "account_history_id_type", "account_statistics_id_type",
    "account_transaction_history_id_type",
    "asset_bitasset_data_id_type", "asset_dynamic_data_id_type",
    "block_summary_id_type", "budget_record_id_type",
    "chain_property_id_type", "fba_accumulator_id_type",
    "global_property_id_type", "dynamic_global_property_id_type",
    "special_authority_id_type", "transaction_history_id_type",
    "transaction_id_type", "blinded_balance_id_type",
    "operation_history_id_type",
}
for _t in _CHAIN_ID_TYPES:
    PRIMITIVE_MAP.setdefault(_t, {"type": "string", "pattern": r"^\d+\.\d+\.\d+$"})

_ID_TYPES = {
    "account_id_type", "asset_id_type", "force_settlement_id_type",
    "committee_member_id_type", "witness_id_type", "limit_order_id_type",
    "call_order_id_type", "custom_id_type", "proposal_id_type",
    "operation_history_id_type", "withdraw_permission_id_type",
    "vesting_balance_id_type", "worker_id_type", "balance_id_type",
    "htlc_id_type", "custom_authority_id_type", "ticket_id_type",
    "liquidity_pool_id_type", "samet_fund_id_type", "credit_offer_id_type",
    "credit_deal_id_type",
}
for _t in _ID_TYPES:
    PRIMITIVE_MAP.setdefault(_t, {"type": "string", "pattern": r"^\d+\.\d+\.\d+$"})


def _ref_name_for(cpp_type: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]", "_", cpp_type).strip("_")


@dataclass
class Registry:
    structs: dict[str, StructInfo]
    enums: dict[str, EnumInfo]
    aliases: dict[str, AliasInfo]
    variants: dict[str, VariantInfo]
    reflected_structs: dict[str, ReflectedStruct]
    reflected_enums: dict[str, ReflectedEnum]
    reflected_typenames: set[str]


def _strip_namespace(qname: str) -> str:
    # Return shortest tail (e.g. graphene::protocol::bitasset_options -> bitasset_options
    # but graphene::protocol::bitasset_options::ext -> bitasset_options::ext)
    parts = qname.split("::")
    # Drop graphene::protocol / graphene::chain / graphene::wallet / graphene::app / fc / std prefixes
    for prefix in (
        ["graphene", "protocol"],
        ["graphene", "chain"],
        ["graphene", "wallet"],
        ["graphene", "app"],
        ["graphene"],
        ["fc"],
        ["std"],
    ):
        if parts[:len(prefix)] == prefix:
            parts = parts[len(prefix):]
            break
    return "::".join(parts)


def _lookup_qualified(reg: Registry, name: str, *, owner: str | None = None) -> str | None:
    """Given a possibly-unqualified type name (e.g. ``ext`` or
    ``bitasset_options::ext``), return the fully qualified name from the
    registry that matches it.

    If ``owner`` is supplied, lookups for bare/un-namespaced names prefer
    nested types that share that owner's scope — so the ``ext`` inside
    ``bitasset_options::extension<ext>`` resolves to ``bitasset_options::ext``
    instead of some other ``*::ext`` defined elsewhere.
    """
    # Direct hit on an exact qualified match
    sources_all = (
        reg.reflected_structs, reg.reflected_enums,
        reg.aliases, reg.variants, reg.structs,
    )
    if owner and "::" not in name:
        # Owner scope: walk outward through enclosing scopes
        scope_parts = owner.split("::")
        while scope_parts:
            candidate = "::".join(scope_parts + [name])
            for src_map in sources_all:
                if candidate in src_map:
                    return candidate
            if candidate in reg.reflected_typenames:
                return candidate
            scope_parts.pop()

    candidates: list[str] = []
    for src_map in (reg.reflected_structs, reg.reflected_enums):
        for qname in src_map:
            if qname == name or qname.endswith("::" + name):
                candidates.append(qname)
    for src_map in (reg.aliases, reg.variants):
        for qname in src_map:
            if qname == name or qname.endswith("::" + name):
                candidates.append(qname)
    if reg.reflected_typenames:
        for qname in reg.reflected_typenames:
            if qname == name or qname.endswith("::" + name):
                candidates.append(qname)
    if not candidates:
        for qname in reg.structs:
            if qname == name or qname.endswith("::" + name):
                candidates.append(qname)
    if not candidates:
        return None
    # Prefer types sharing the owner namespace (when given), then by library
    # depth (protocol > chain > wallet > app > everything else).
    def priority(n: str) -> tuple[int, int]:
        owner_match = 0
        if owner:
            owner_ns = "::".join(owner.split("::")[:-1])
            if owner_ns and n.startswith(owner_ns + "::"):
                owner_match = -1  # boost: matches owner namespace
        if n.startswith("graphene::protocol::"): lib = 0
        elif n.startswith("graphene::chain::"):  lib = 1
        elif n.startswith("graphene::wallet::"): lib = 2
        elif n.startswith("graphene::app::"):    lib = 3
        else:                                    lib = 4
        return (owner_match, lib)
    candidates.sort(key=priority)
    return candidates[0]


def map_type(cpp_type: str, reg: Registry, pending: dict[str, str],
             *, owner: str | None = None) -> dict:
    """Map a C++ type expression to a JSON Schema fragment.

    ``pending[ref_name] = qualified_cpp_type`` collects references that still
    need a schema generated (resolved by the main loop).

    ``owner`` is the qualified name of the struct currently being expanded.
    It lets unqualified names like ``ext`` inside ``extension<ext>`` resolve
    to ``<owner>::ext`` rather than the first ``*::ext`` in the registry.
    """
    # Collapse whitespace (in particular newlines from multi-line typedefs
    # like `typedef static_variant<\n A,\n B\n> foo;`) so the regexes below
    # don't have to be DOTALL.
    t = re.sub(r"\s+", " ", cpp_type).strip()
    # Drop top-level `const`/`volatile` and reference / pointer markers.
    t = re.sub(r"\bconst\b", "", t)
    t = re.sub(r"\bvolatile\b", "", t)
    t = t.replace("&", "").replace("*", "")
    t = re.sub(r"\s+", " ", t).strip()
    if not t:
        return {}

    # Smart pointers and reference-counted wrappers: pass through to inner type.
    m = re.match(r"^(?:std::)?(?:shared_ptr|unique_ptr|weak_ptr)\s*<(.+)>\s*$", t)
    if m:
        return map_type(m.group(1), reg, pending, owner=owner)

    # `X::flat_set_type` is the nested typedef that `fc::static_variant` defines
    # as `flat_set<self>` — serialized like a flat_set of the variant itself.
    m = re.match(r"^(.+?)::flat_set_type$", t)
    if m:
        return {"type": "array", "items": map_type(m.group(1), reg, pending, owner=owner), "uniqueItems": True}

    # `transform_to_fee_parameters<V>::type` is a template metafunction that
    # maps each alternative of variant V to its `fee_params_t` nested type.
    # We synthesize the equivalent static_variant by name.
    m = re.match(r"^transform_to_fee_parameters\s*<(.+?)>::type$", t)
    if m:
        variant_name = m.group(1).strip()
        var_qname = _lookup_qualified(reg, variant_name)
        if var_qname is not None and var_qname in reg.variants:
            alts = [f"{a}::fee_params_t" for a in reg.variants[var_qname].alternatives]
            return _variant_schema_from_alternatives(alts, reg, pending)

    # Inline static_variant<...> — produces a [index, payload] oneOf
    sv = _match_static_variant(t)
    if sv is not None:
        return _variant_schema_from_alternatives(sv, reg, pending, owner=owner)

    # Direct primitive
    if t in PRIMITIVE_MAP:
        return dict(PRIMITIVE_MAP[t])

    # Containers
    m = re.match(r"^([A-Za-z_][\w:]*)\s*<(.+)>\s*$", t)
    if m:
        outer = m.group(1)
        outer_stripped = outer.replace("std::", "").replace("fc::", "")
        inner = _split_template_args(m.group(2))
        if outer_stripped in ("vector", "deque", "list") and len(inner) == 1:
            return {"type": "array", "items": map_type(inner[0], reg, pending, owner=owner)}
        if outer_stripped in ("set", "flat_set", "unordered_set") and len(inner) == 1:
            return {"type": "array", "items": map_type(inner[0], reg, pending, owner=owner), "uniqueItems": True}
        if outer_stripped == "optional" and len(inner) == 1:
            inner_schema = map_type(inner[0], reg, pending, owner=owner)
            if "$ref" in inner_schema:
                return {"oneOf": [inner_schema, {"type": "null"}]}
            out = dict(inner_schema)
            ty = out.get("type")
            if isinstance(ty, str):
                out["type"] = [ty, "null"]
            elif isinstance(ty, list) and "null" not in ty:
                out["type"] = ty + ["null"]
            else:
                return {"oneOf": [inner_schema, {"type": "null"}]}
            return out
        if outer_stripped == "safe" and len(inner) == 1:
            return map_type(inner[0], reg, pending, owner=owner)
        # `extension<T>` wraps a struct T whose fields are all optional<>.
        # Serialized JSON is just T's object form.
        if outer_stripped == "extension" and len(inner) == 1:
            return map_type(inner[0], reg, pending, owner=owner)
        if outer_stripped in ("map", "flat_map", "unordered_map") and len(inner) >= 2:
            return {"type": "object", "additionalProperties": map_type(inner[1], reg, pending, owner=owner)}
        if outer_stripped == "pair" and len(inner) == 2:
            return {
                "type": "array",
                "prefixItems": [map_type(a, reg, pending, owner=owner) for a in inner],
                "minItems": 2,
                "maxItems": 2,
            }
        if outer_stripped == "tuple":
            return {
                "type": "array",
                "prefixItems": [map_type(a, reg, pending, owner=owner) for a in inner],
                "minItems": len(inner),
                "maxItems": len(inner),
            }
        # Unknown generic — emit a placeholder for the full expression
        ref = _ref_name_for(t)
        pending.setdefault(ref, t)
        return {"$ref": f"#/components/schemas/{ref}"}

    # Bare type: look up in registry by suffix
    qualified = _lookup_qualified(reg, t, owner=owner)
    if qualified is None:
        ref = _ref_name_for(t)
        pending.setdefault(ref, t)
        return {"$ref": f"#/components/schemas/{ref}"}

    ref = _ref_name_for(_strip_namespace(qualified))
    pending.setdefault(ref, qualified)
    return {"$ref": f"#/components/schemas/{ref}"}


# ---------------------------------------------------------------------------
# Schema synthesis for a single reflected type
# ---------------------------------------------------------------------------

def _all_field_names(reflected: ReflectedStruct, reg: Registry) -> list[tuple[str, str]]:
    """Return [(field_name, qualified_struct_owning_field)] including inherited
    fields from FC_REFLECT_DERIVED base list (recursive)."""
    out: list[tuple[str, str]] = []
    visited: set[str] = set()

    def visit(qname: str) -> None:
        if qname in visited:
            return
        visited.add(qname)
        node = reg.reflected_structs.get(qname)
        if node is None:
            return
        for base in node.base_names:
            visit(base)
        for fname in node.field_names:
            out.append((fname, qname))

    visit(reflected.qualified_name)
    return out


def schema_for_struct(reflected: ReflectedStruct, reg: Registry,
                      pending: dict[str, str]) -> dict:
    properties: dict[str, dict] = {}
    required: list[str] = []
    missing: list[str] = []
    for fname, owner in _all_field_names(reflected, reg):
        struct = reg.structs.get(owner)
        ftype: str | None = None
        if struct is not None:
            ftype = find_field_type(struct.body, fname)
        if not ftype:
            # Couldn't locate declaration — leave a stub but mark TODO
            properties[fname] = {
                "x-cpp-type": "?",
                "description": f"TODO: could not resolve declaration of {fname} in {owner}",
            }
            missing.append(fname)
            required.append(fname)
            continue
        fschema = map_type(ftype, reg, pending, owner=owner)
        fschema_out = dict(fschema)
        fschema_out["x-cpp-type"] = ftype
        properties[fname] = fschema_out
        required.append(fname)

    schema: dict = {
        "type": "object",
        "x-cpp-type": reflected.qualified_name,
        "properties": properties,
    }
    if required:
        schema["required"] = required
    if missing:
        schema["x-unresolved-fields"] = missing
    return schema


def schema_for_enum(en: ReflectedEnum, reg: Registry) -> dict:
    # Enum is serialized as a string when scoped, sometimes as int otherwise.
    # We emit { type: string, enum: [...] } since FC serializes by name in JSON.
    return {
        "x-cpp-type": en.qualified_name,
        "type": "string",
        "enum": list(en.values),
    }


def _variant_schema_from_alternatives(alts: list[str], reg: Registry,
                                       pending: dict[str, str],
                                       *, owner: str | None = None) -> dict:
    one_of: list[dict] = []
    for idx, alt in enumerate(alts):
        alt_clean = re.sub(r"\s+", " ", alt).strip()
        alt_schema = map_type(alt_clean, reg, pending, owner=owner)
        one_of.append({
            "type": "array",
            "prefixItems": [
                {"const": idx, "description": alt_clean},
                alt_schema,
            ],
            "minItems": 2,
            "maxItems": 2,
        })
    return {
        "description": f"Discriminated union — JSON form is [index, value]. {len(alts)} alternatives.",
        "oneOf": one_of,
    }


def schema_for_variant(var: VariantInfo, reg: Registry, pending: dict[str, str]) -> dict:
    """static_variant — JSON form is [tag_index_or_name, value]. Bitshares uses
    [int_index, struct]. We model it as a heterogeneous tuple union."""
    schema = _variant_schema_from_alternatives(var.alternatives, reg, pending)
    schema["x-cpp-type"] = var.qualified_name
    return schema


# ---------------------------------------------------------------------------
# Main: load spec, fill schemas, write back
# ---------------------------------------------------------------------------

def collect_headers(roots: list[Path]) -> list[Path]:
    """Gather .hpp and .cpp files under each root.

    .cpp files are included because many chain objects use
    FC_REFLECT_DERIVED_NO_TYPENAME from their .cpp rather than their header.
    """
    out: list[Path] = []
    for root in roots:
        if root.is_file():
            out.append(root)
            continue
        for ext in ("*.hpp", "*.cpp"):
            for p in sorted(root.rglob(ext)):
                out.append(p)
    return out


def build_registry(headers: list[Path]) -> Registry:
    structs: dict[str, StructInfo] = {}
    enums: dict[str, EnumInfo] = {}
    aliases: dict[str, AliasInfo] = {}
    variants: dict[str, VariantInfo] = {}
    reflected_structs: dict[str, ReflectedStruct] = {}
    reflected_enums: dict[str, ReflectedEnum] = {}
    reflected_typenames: set[str] = set()

    for path in headers:
        try:
            raw = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        src = preprocess(raw)
        parse_cpp(src, str(path), structs=structs, enums=enums,
                  aliases=aliases, variants=variants)
        rs, re_, rt = parse_reflect_macros(src)
        for r in rs:
            reflected_structs[r.qualified_name] = r
        for r in re_:
            reflected_enums[r.qualified_name] = r
        for r in rt:
            reflected_typenames.add(r.qualified_name)

    return Registry(
        structs=structs, enums=enums, aliases=aliases, variants=variants,
        reflected_structs=reflected_structs, reflected_enums=reflected_enums,
        reflected_typenames=reflected_typenames,
    )


def fill_schemas(spec: dict, reg: Registry) -> tuple[int, list[str]]:
    schemas: dict[str, dict] = spec.setdefault("components", {}).setdefault("schemas", {})

    # Start with existing placeholders — their `x-cpp-type` carries the type
    # name to resolve.
    worklist: dict[str, str] = {}
    for ref_name, schema in schemas.items():
        if isinstance(schema, dict) and schema.get("description", "").startswith("TODO"):
            cpp_type = schema.get("x-cpp-type", ref_name)
            worklist[ref_name] = cpp_type

    resolved = 0
    unresolved: list[str] = []

    seen: set[str] = set()
    while worklist:
        ref_name, cpp_type = worklist.popitem()
        if ref_name in seen:
            continue
        seen.add(ref_name)

        new_pending: dict[str, str] = {}

        # 1. variant?
        qname = _lookup_qualified(reg, cpp_type)
        if qname is not None and qname in reg.variants:
            schemas[ref_name] = schema_for_variant(reg.variants[qname], reg, new_pending)
            resolved += 1
        # 2. alias?
        elif qname is not None and qname in reg.aliases:
            aliased = reg.aliases[qname].cpp_type
            mapped = map_type(aliased, reg, new_pending)
            mapped.setdefault("x-cpp-type", qname)
            mapped.setdefault("description", f"Alias for {aliased}")
            schemas[ref_name] = mapped
            resolved += 1
        # 3. enum?
        elif qname is not None and qname in reg.reflected_enums:
            schemas[ref_name] = schema_for_enum(reg.reflected_enums[qname], reg)
            resolved += 1
        # 4. struct?
        elif qname is not None and qname in reg.reflected_structs:
            schemas[ref_name] = schema_for_struct(reg.reflected_structs[qname], reg, new_pending)
            resolved += 1
        else:
            # Leave the placeholder, mark unresolved
            unresolved.append(f"{ref_name} ({cpp_type})")
            schemas[ref_name].setdefault("x-cpp-type", cpp_type)

        # Pull in transitively-referenced types
        for new_ref, new_type in new_pending.items():
            if new_ref in schemas:
                # Already present — only enqueue if still a placeholder
                existing = schemas[new_ref]
                if isinstance(existing, dict) and existing.get("description", "").startswith("TODO"):
                    worklist.setdefault(new_ref, new_type)
                continue
            # First time we see it — register a placeholder, enqueue
            schemas[new_ref] = {
                "x-cpp-type": new_type,
                "description": "TODO: fill from FC_REFLECT walker",
            }
            worklist[new_ref] = new_type

    return resolved, unresolved


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--spec", required=True, type=Path,
                    help="OpenRPC document to enrich (read & write).")
    ap.add_argument("--output", type=Path, default=None,
                    help="Write enriched document here (default: overwrite --spec).")
    ap.add_argument("--header-root", action="append", type=Path, required=True,
                    help="Directory (recursed) or file to scan. Pass repeatedly.")
    args = ap.parse_args()

    headers = collect_headers(args.header_root)
    if not headers:
        sys.exit("No headers found under provided --header-root paths.")
    reg = build_registry(headers)

    spec = json.loads(args.spec.read_text(encoding="utf-8"))
    resolved, unresolved = fill_schemas(spec, reg)

    out_path = args.output or args.spec
    out_path.write_text(json.dumps(spec, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    print(
        f"Filled {resolved} schemas; "
        f"reflected types in registry: {len(reg.reflected_structs)} structs, "
        f"{len(reg.reflected_enums)} enums, {len(reg.variants)} variants, "
        f"{len(reg.aliases)} aliases.",
        file=sys.stderr,
    )
    if unresolved:
        print(f"Unresolved placeholders ({len(unresolved)}):", file=sys.stderr)
        for u in unresolved:
            print(f"  {u}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
