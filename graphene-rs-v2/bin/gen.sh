#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

rm -rf target/generated

for chain in bitshares acta rsquared swaplock; do
  cargo run -p graphene-codegen --bin graphene-codegen -- \
    generate "chains/${chain}.toml"
  cargo run -p graphene-codegen --bin graphene-codegen -- \
    audit "chains/${chain}.toml"
done

cargo fmt
cargo test
