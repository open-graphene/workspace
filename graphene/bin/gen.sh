#!/usr/bin/env bash
# Generate open-graphene specs for the currently supported non-BitShares chains.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GRAPHENE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
WORKSPACE_ROOT="$(cd "${GRAPHENE_DIR}/.." && pwd)"

cd "${WORKSPACE_ROOT}"

CHAINS=(
  "swaplock"
  "acta"
)

for chain in "${CHAINS[@]}"; do
  config="graphene/chains/${chain}.toml"
  echo "==> Generating ${chain} open-graphene specs"
  cargo run -p graphene-gen-spec -- generate "${config}"
  echo "==> Auditing ${chain} open-graphene specs"
  cargo run -p graphene-gen-spec -- audit "${config}"
done

echo "==> Specs written under graphene/specs"
