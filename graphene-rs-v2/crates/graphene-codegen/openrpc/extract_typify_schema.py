#!/usr/bin/env python3
"""Extract OpenRPC components.schemas into a JSON Schema document for typify.

OpenRPC stores reusable schemas under `components.schemas` and references them as
`#/components/schemas/Name`. Typify expects a JSON Schema document with `$defs`
and refs like `#/$defs/Name`. This script performs that normalization.

It intentionally does not run typify. The next v2 step can either call typify as
an explicit Rust generator stage or use the emitted schema document in a chain
crate build experiment.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def rewrite_refs(value: Any) -> Any:
    if isinstance(value, dict):
        result = {}
        for key, item in value.items():
            if key == "$ref" and isinstance(item, str):
                if item.startswith("#/components/schemas/"):
                    result[key] = "#/$defs/" + item.rsplit("/", 1)[-1]
                else:
                    result[key] = item
            else:
                result[key] = rewrite_refs(item)
        return result
    if isinstance(value, list):
        return [rewrite_refs(item) for item in value]
    return value


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", required=True, type=Path, help="OpenRPC spec JSON path")
    parser.add_argument("--output", required=True, type=Path, help="JSON Schema document to write")
    parser.add_argument("--title", default="graphene_chain_types_root", help="root schema title")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if not args.spec.exists():
        sys.exit(f"spec not found: {args.spec}")

    spec = json.loads(args.spec.read_text(encoding="utf-8"))
    schemas = spec.get("components", {}).get("schemas")
    if not isinstance(schemas, dict):
        sys.exit("OpenRPC spec is missing components.schemas")

    schema_doc = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": args.title,
        "$defs": rewrite_refs(schemas),
    }

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(schema_doc, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"Wrote {args.output} ({len(schemas)} defs)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
