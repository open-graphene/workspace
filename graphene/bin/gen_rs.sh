#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_DIR="$(cd "${WORKSPACE_DIR}/.." && pwd)"
ROOT_OUT_DIR="${REPO_DIR}/graphene-rs"
TMP_DIR="${WORKSPACE_DIR}/.tmp-rs-sdk"

rm -rf "${ROOT_OUT_DIR}"
mkdir -p "${ROOT_OUT_DIR}"

for CHAIN in swaplock acta; do
  TMP_CHAIN_DIR="${TMP_DIR}-${CHAIN}"
  OUT_DIR="${ROOT_OUT_DIR}/graphene-bindings-${CHAIN}"

  rm -rf "${TMP_CHAIN_DIR}"
  mkdir -p "${TMP_CHAIN_DIR}"

  (
    cd "${WORKSPACE_DIR}"
    cargo run -p graphene-spec-gen -- generate \
      "specs/${CHAIN}.opengraphene.json" \
      --openrpc "crates/graphene-spec-gen/fixtures/${CHAIN}.openrpc.json" \
      --target rust \
      --out "${TMP_CHAIN_DIR}"
  )

  mkdir -p "${OUT_DIR}/src"
  cp "${TMP_CHAIN_DIR}/rust/src/lib.rs" "${OUT_DIR}/src/lib.rs"
  cat > "${OUT_DIR}/Cargo.toml" <<EOF
[package]
name = "graphene-bindings-${CHAIN}"
edition = "2021"
version = "0.1.0"
publish = false

[lib]
path = "src/lib.rs"

[workspace]
EOF
  rm -rf "${TMP_CHAIN_DIR}"
  echo "regenerated Rust Graphene SDK artifacts in ${OUT_DIR}"
done
