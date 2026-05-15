#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_DIR="$(cd "${WORKSPACE_DIR}/.." && pwd)"
RUST_WORKSPACE_DIR="${REPO_DIR}/graphene-rs"
ROOT_OUT_DIR="${RUST_WORKSPACE_DIR}/crates"
OPENRPC_BACKEND_DIR="${WORKSPACE_DIR}/crates/graphene-gen-rust/openrpc"
FULL_OPENRPC_DIR="${WORKSPACE_DIR}/crates/graphene-gen-rust/fixtures/openrpc-full"
TMP_DIR="${WORKSPACE_DIR}/.tmp-rs-sdk"

mkdir -p "${ROOT_OUT_DIR}"

for CHAIN in swaplock acta; do
  TMP_CHAIN_DIR="${TMP_DIR}-${CHAIN}"
  OUT_DIR="${ROOT_OUT_DIR}/graphene-bindings-${CHAIN}"
  GENERATED_DIR="${OUT_DIR}/src/generated"
  DATABASE_SPEC="${FULL_OPENRPC_DIR}/${CHAIN}.database.openrpc.json"
  BROADCAST_SPEC="${FULL_OPENRPC_DIR}/${CHAIN}.broadcast.openrpc.json"

  rm -rf "${TMP_CHAIN_DIR}"
  mkdir -p "${TMP_CHAIN_DIR}" "${GENERATED_DIR}"

  python3 "${OPENRPC_BACKEND_DIR}/extract_typify_schema.py" \
    --spec "${DATABASE_SPEC}" \
    --output "${TMP_CHAIN_DIR}/database.schema.json" \
    --title "${CHAIN}_types_root"

  python3 "${OPENRPC_BACKEND_DIR}/gen_rust_variants.py" \
    --spec "${DATABASE_SPEC}" \
    --out-rs "${GENERATED_DIR}/variants.rs" \
    --out-names "${GENERATED_DIR}/variants.names" \
    --source-label "crates/graphene-gen-rust/fixtures/openrpc-full/${CHAIN}.database.openrpc.json"

  python3 "${OPENRPC_BACKEND_DIR}/gen_rust_rpc.py" \
    --spec "${DATABASE_SPEC}" \
    --out-rs "${GENERATED_DIR}/rpc.rs" \
    --source-label "crates/graphene-gen-rust/fixtures/openrpc-full/${CHAIN}.database.openrpc.json"

  (
    cd "${WORKSPACE_DIR}"
    cargo run --quiet -p graphene-gen-rust --bin openrpc-typify -- \
      --schema "${TMP_CHAIN_DIR}/database.schema.json" \
      --out "${GENERATED_DIR}/types.rs" \
      --strip-names "${GENERATED_DIR}/variants.names"
  )

  python3 "${OPENRPC_BACKEND_DIR}/gen_rust_rpc.py" \
    --spec "${BROADCAST_SPEC}" \
    --out-rs "${GENERATED_DIR}/broadcast_rpc.rs" \
    --source-label "crates/graphene-gen-rust/fixtures/openrpc-full/${CHAIN}.broadcast.openrpc.json"

  (
    cd "${WORKSPACE_DIR}"
    cargo run -p graphene-gen-rust --bin graphene-gen-rust-spec-metadata -- \
      "specs/${CHAIN}.opengraphene.json" \
      --openrpc "crates/graphene-spec-gen/fixtures/${CHAIN}.openrpc.json" \
      --out "${TMP_CHAIN_DIR}/spec_metadata.rs"
  )

  cp "${TMP_CHAIN_DIR}/spec_metadata.rs" "${OUT_DIR}/src/spec_metadata.rs"
  rm -rf "${TMP_CHAIN_DIR}"
  echo "regenerated Rust Graphene bindings in ${OUT_DIR}"
done

cargo fmt --manifest-path "${RUST_WORKSPACE_DIR}/Cargo.toml" --all
