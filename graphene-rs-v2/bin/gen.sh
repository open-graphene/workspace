#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

rm -rf target/generated/swaplock

cargo run -p graphene-codegen --bin graphene-codegen -- \
  generate chains/swaplock.toml

cargo fmt
cargo test
