#!/usr/bin/env bash
# Generate Rust data-model bindings from the generated open-graphene specs.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GRAPHENE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
WORKSPACE_ROOT="$(cd "${GRAPHENE_DIR}/.." && pwd)"

cd "${WORKSPACE_ROOT}"

echo "==> Generating Rust bindings for open-graphene specs"
cargo run -p graphene-gen-bindings-rs --bin graphene-gen-bindings-rs -- --chain all

echo "==> Verifying Rust binding crates"
cargo test --manifest-path graphene-rs/Cargo.toml

echo "==> Rust bindings written under graphene-rs/crates"
