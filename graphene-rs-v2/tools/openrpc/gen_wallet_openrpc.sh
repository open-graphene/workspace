#!/usr/bin/env bash
# Generate a Graphene wallet OpenRPC specification for any Graphene-like chain.
#
# This is the generic version of the Swaplock POC pipeline:
#   1. Run Doxygen against wallet.hpp.
#   2. Extract wallet_api methods into OpenRPC.
#   3. Fill components.schemas by walking FC_REFLECT/static_variant macros.
#
# Usage:
#   gen_wallet_openrpc.sh \
#     --chain-name swaplock \
#     --core-root /path/to/swaplock-core \
#     --output /path/to/spec.json
#
# Optional:
#   --wallet-header relative/or/absolute/path/to/wallet.hpp
#   --header-root relative/or/absolute/path (repeatable)
#   --build-dir /tmp/graphene-wallet-spec

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHAIN_NAME=""
CORE_ROOT=""
OUTPUT_FILE=""
WALLET_HEADER="libraries/wallet/include/graphene/wallet/wallet.hpp"
BUILD_DIR=""
HEADER_ROOTS=()

die() { echo "gen_wallet_openrpc: $*" >&2; exit 1; }
usage() {
  cat >&2 <<'EOF'
Generate a Graphene wallet OpenRPC specification.

Usage:
  gen_wallet_openrpc.sh \
    --chain-name <name> \
    --core-root <path-to-chain-core> \
    --output <path-to-spec.json>

Optional:
  --wallet-header <relative-or-absolute-wallet.hpp>
  --header-root <relative-or-absolute-header-root>  # repeatable
  --build-dir <path>
EOF
}

resolve_path() {
  local base="$1"
  local path="$2"
  if [[ "$path" = /* ]]; then
    printf '%s\n' "$path"
  else
    printf '%s\n' "$base/$path"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --chain-name)
      [[ $# -ge 2 ]] || die "--chain-name requires a value"
      CHAIN_NAME="$2"
      shift 2
      ;;
    --core-root)
      [[ $# -ge 2 ]] || die "--core-root requires a path"
      CORE_ROOT="$2"
      shift 2
      ;;
    --output)
      [[ $# -ge 2 ]] || die "--output requires a path"
      OUTPUT_FILE="$2"
      shift 2
      ;;
    --wallet-header)
      [[ $# -ge 2 ]] || die "--wallet-header requires a path"
      WALLET_HEADER="$2"
      shift 2
      ;;
    --header-root)
      [[ $# -ge 2 ]] || die "--header-root requires a path"
      HEADER_ROOTS+=("$2")
      shift 2
      ;;
    --build-dir)
      [[ $# -ge 2 ]] || die "--build-dir requires a path"
      BUILD_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

[[ -n "$CHAIN_NAME" ]] || die "missing --chain-name"
[[ -n "$CORE_ROOT" ]] || die "missing --core-root"
[[ -n "$OUTPUT_FILE" ]] || die "missing --output"

CORE_ROOT="$(cd "$CORE_ROOT" && pwd)"
WALLET_HEADER="$(resolve_path "$CORE_ROOT" "$WALLET_HEADER")"
OUTPUT_FILE="$(python3 -c 'import os,sys; print(os.path.abspath(sys.argv[1]))' "$OUTPUT_FILE")"
BUILD_DIR="${BUILD_DIR:-${CORE_ROOT}/build/wallet-openrpc}"
XML_DIR="${BUILD_DIR}/xml"

if [[ ${#HEADER_ROOTS[@]} -eq 0 ]]; then
  HEADER_ROOTS=(
    "libraries/protocol"
    "libraries/chain"
    "libraries/app"
    "libraries/wallet"
    "libraries/plugins"
  )
fi

command -v doxygen >/dev/null 2>&1 || die "doxygen not found. Install with: brew install doxygen (macOS) or apt-get install doxygen (Linux)."
command -v python3 >/dev/null 2>&1 || die "python3 not found."
[[ -f "$WALLET_HEADER" ]] || die "wallet header not found: $WALLET_HEADER"

rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR" "$(dirname "$OUTPUT_FILE")"

cat > "${BUILD_DIR}/Doxyfile" <<EOF
PROJECT_NAME           = "${CHAIN_NAME} wallet_api"
OUTPUT_DIRECTORY       = ${BUILD_DIR}
INPUT                  = ${WALLET_HEADER}
INPUT_ENCODING         = UTF-8
RECURSIVE              = NO
QUIET                  = YES
WARNINGS               = NO
WARN_IF_UNDOCUMENTED   = NO
EXTRACT_ALL            = YES
EXTRACT_PRIVATE        = NO
EXTRACT_STATIC         = NO
EXTRACT_LOCAL_CLASSES  = YES
EXTRACT_LOCAL_METHODS  = NO
GENERATE_HTML          = NO
GENERATE_LATEX         = NO
GENERATE_XML           = YES
XML_OUTPUT             = xml
XML_PROGRAMLISTING     = NO
MACRO_EXPANSION        = NO
EOF

doxygen "${BUILD_DIR}/Doxyfile" >/dev/null
[[ -d "$XML_DIR" ]] || die "Doxygen produced no XML at $XML_DIR"

python3 "${SCRIPT_DIR}/gen_wallet_spec.py" \
  --xml-dir "$XML_DIR" \
  --wallet-header "$WALLET_HEADER" \
  > "$OUTPUT_FILE"

TYPE_ARGS=(--spec "$OUTPUT_FILE")
for root in "${HEADER_ROOTS[@]}"; do
  resolved="$(resolve_path "$CORE_ROOT" "$root")"
  [[ -e "$resolved" ]] || die "header root not found: $resolved"
  TYPE_ARGS+=(--header-root "$resolved")
done
python3 "${SCRIPT_DIR}/gen_types_spec.py" "${TYPE_ARGS[@]}"

method_count=$(python3 -c "import json,sys; print(len(json.load(open(sys.argv[1]))['methods']))" "$OUTPUT_FILE")
schema_count=$(python3 -c "import json,sys; print(len(json.load(open(sys.argv[1]))['components']['schemas']))" "$OUTPUT_FILE")
echo "Wrote ${OUTPUT_FILE} (${method_count} methods, ${schema_count} schemas)"
