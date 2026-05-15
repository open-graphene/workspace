#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
GRAPHENE_RS_DIR="$(cd -- "$CRATE_DIR/../.." && pwd)"

cargo run \
  --manifest-path "$GRAPHENE_RS_DIR/Cargo.toml" \
  -p graphene-codegen \
  --bin generate_object_ids \
  -- --config "$CRATE_DIR/codegen.toml" "$@"
