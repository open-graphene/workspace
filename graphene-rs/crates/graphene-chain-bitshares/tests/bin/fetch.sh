#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
TESTS_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
CRATE_DIR="$(cd -- "$TESTS_DIR/.." && pwd)"
TYPES_DIR="$CRATE_DIR/src/types"
FIXTURES_DIR="${BITSHARES_FIXTURES_DIR:-$TESTS_DIR/fixtures}"
ENDPOINT="${BITSHARES_RPC_ENDPOINT:-wss://cloud.xbts.io/ws}"
MAX_PROBE="${BITSHARES_FIXTURE_MAX_PROBE:-100}"

case "$ENDPOINT" in
  wss://*) ENDPOINT="https://${ENDPOINT#wss://}" ;;
  ws://*) ENDPOINT="http://${ENDPOINT#ws://}" ;;
esac

mkdir -p "$FIXTURES_DIR"

if ! command -v curl >/dev/null 2>&1; then
  echo "error: curl is required" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "error: python3 is required" >&2
  exit 1
fi

if [[ ! "$MAX_PROBE" =~ ^[0-9]+$ ]]; then
  echo "error: BITSHARES_FIXTURE_MAX_PROBE must be a non-negative integer" >&2
  exit 1
fi

selected_families=("$@")

family_selected() {
  local family="$1"
  if ((${#selected_families[@]} == 0)); then
    return 0
  fi

  local selected
  for selected in "${selected_families[@]}"; do
    if [[ "$selected" == "$family" ]]; then
      return 0
    fi
  done
  return 1
}

while IFS=$'\t' read -r family prefix; do
  if ! family_selected "$family"; then
    continue
  fi

  output_path="$FIXTURES_DIR/$family.json"
  request_path="$(mktemp)"
  response_path="$(mktemp)"
  trap 'rm -f "$request_path" "$response_path"' EXIT

  python3 - "$prefix" "$MAX_PROBE" >"$request_path" <<'PY'
import json
import sys

prefix = sys.argv[1]
max_probe = int(sys.argv[2])
ids = [f"{prefix}.{instance}" for instance in range(max_probe + 1)]
print(json.dumps({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call",
    "params": [0, "get_objects", [ids]],
}, separators=(",", ":")))
PY

  curl \
    --silent \
    --show-error \
    --fail \
    --max-time "${BITSHARES_RPC_TIMEOUT_SECONDS:-30}" \
    --header 'content-type: application/json' \
    --data "@$request_path" \
    "$ENDPOINT" \
    >"$response_path"

  python3 - "$family" "$prefix" "$ENDPOINT" "$MAX_PROBE" "$response_path" "$output_path" <<'PY'
import json
import sys
from pathlib import Path

family, prefix, endpoint, max_probe, response_path, output_path = sys.argv[1:]
max_probe = int(max_probe)
response = json.loads(Path(response_path).read_text())

envelope = {
    "endpoint": endpoint,
    "family": family,
    "object_id_prefix": prefix,
    "probe_range": [0, max_probe],
    "object_id": None,
    "object": None,
}

if "error" in response:
    envelope["error"] = response["error"]
    Path(output_path).write_text(json.dumps(envelope, indent=2, sort_keys=True) + "\n")
    print(f"{family}: RPC error -> {output_path}")
    raise SystemExit(0)

results = response.get("result")
if not isinstance(results, list):
    envelope["error"] = {"message": "unexpected RPC response", "response": response}
    Path(output_path).write_text(json.dumps(envelope, indent=2, sort_keys=True) + "\n")
    print(f"{family}: unexpected response -> {output_path}")
    raise SystemExit(0)

found = next((item for item in results if item is not None), None)
envelope["object_id"] = found.get("id") if isinstance(found, dict) else None
envelope["object"] = found
Path(output_path).write_text(json.dumps(envelope, indent=2, sort_keys=True) + "\n")
print(f"{family}: {envelope['object_id'] or 'not found'} -> {output_path}")
PY

  rm -f "$request_path" "$response_path"
  trap - EXIT
done < <(
  python3 - "$TYPES_DIR" <<'PY'
import re
import sys
from pathlib import Path

source_re = re.compile(r"^// Source family: ([a-z0-9_]+) \((\d+\.\d+)\.x\)$")
types_dir = Path(sys.argv[1])
rows = []
for path in sorted(types_dir.glob("*.rs")):
    if path.name == "mod.rs":
        continue
    for line in path.read_text().splitlines():
        match = source_re.match(line)
        if match:
            rows.append((match.group(1), match.group(2)))
            break
for family, prefix in sorted(rows):
    print(f"{family}\t{prefix}")
PY
)
