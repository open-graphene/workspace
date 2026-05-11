#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
GRAPHENE_RS_DIR="$(cd -- "$SCRIPT_DIR/../../.." && pwd)"

for GEN_SCRIPT in "$GRAPHENE_RS_DIR"/crates/graphene-chain-*/bin/gen.sh; do
  if [[ -x "$GEN_SCRIPT" ]]; then
    echo "==> ${GEN_SCRIPT#$GRAPHENE_RS_DIR/}"
    "$GEN_SCRIPT" "$@"
  fi
done
