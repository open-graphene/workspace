#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
SPECS_DIR="${WORKSPACE_DIR}/specs"
TMP_DIR="${WORKSPACE_DIR}/.tmp-specs"

rm -rf "${TMP_DIR}"
mkdir -p "${TMP_DIR}"

(
  cd "${WORKSPACE_DIR}"
  cargo run -p open-graphene-gen -- opengraphene-specs --out "${TMP_DIR}"
)

rm -rf "${SPECS_DIR}"
mv "${TMP_DIR}" "${SPECS_DIR}"

echo "regenerated OpenGraphene specs in ${SPECS_DIR}"
