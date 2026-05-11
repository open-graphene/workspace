#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
GRAPHENE_V2_DIR="$(cd -- "$CRATE_DIR/../../.." && pwd)"
REPO_DIR="$(cd -- "$GRAPHENE_V2_DIR/.." && pwd)"

"$SCRIPT_DIR/gen.sh" --write

git -C "$REPO_DIR" diff --exit-code -- \
  "graphene-v2/graphene-rs/crates/graphene-chain-bitshares/src/operation_variants.rs" \
  "graphene-v2/graphene-rs/crates/graphene-chain-bitshares/src/operations.rs" \
  "graphene-v2/graphene-rs/crates/graphene-chain-bitshares/src/operation_model_skips.md" \
  "graphene-v2/graphene-rs/crates/graphene-chain-bitshares/src/types"
