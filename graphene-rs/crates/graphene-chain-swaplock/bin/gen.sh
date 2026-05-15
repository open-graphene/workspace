#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
GRAPHENE_RS_DIR="$(cd -- "$CRATE_DIR/../.." && pwd)"

run_generator() {
  local generator="$1"
  shift

  cargo run \
    --manifest-path "$GRAPHENE_RS_DIR/Cargo.toml" \
    -p graphene-codegen \
    --bin "$generator" \
    -- --config "$CRATE_DIR/codegen.toml" "$@"
}

has_write_arg() {
  for arg in "$@"; do
    if [[ "$arg" == "--write" ]]; then
      return 0
    fi
  done
  return 1
}

run_generator generate_object_ids "$@"
run_generator generate_operation_tags "$@"
run_generator generate_operations "$@"

if has_write_arg "$@"; then
  cargo fmt \
    --manifest-path "$GRAPHENE_RS_DIR/Cargo.toml" \
    -p graphene-chain-swaplock
fi
