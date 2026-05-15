#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
REPO_DIR="$(cd "${WORKSPACE_DIR}/.." && pwd)"
ROOT_OUT_DIR="${REPO_DIR}/graphene-ts"
TMP_DIR="${WORKSPACE_DIR}/.tmp-ts-sdk"

rm -rf "${ROOT_OUT_DIR}"
mkdir -p "${ROOT_OUT_DIR}"

for CHAIN in swaplock acta; do
  TMP_CHAIN_DIR="${TMP_DIR}-${CHAIN}"
  OUT_DIR="${ROOT_OUT_DIR}/graphene-ts-bindings-${CHAIN}"

  rm -rf "${TMP_CHAIN_DIR}"
  mkdir -p "${TMP_CHAIN_DIR}"

  (
    cd "${WORKSPACE_DIR}"
    cargo run -p graphene-spec-gen -- generate \
      "specs/${CHAIN}.opengraphene.json" \
      --openrpc "crates/graphene-spec-gen/fixtures/${CHAIN}.openrpc.json" \
      --target typescript \
      --out "${TMP_CHAIN_DIR}"
  )

  mkdir -p "${OUT_DIR}/src"
  cp "${TMP_CHAIN_DIR}/typescript/index.ts" "${OUT_DIR}/src/index.ts"
  rm -rf "${TMP_CHAIN_DIR}"
  echo "regenerated TypeScript Graphene SDK artifacts in ${OUT_DIR}"
done
