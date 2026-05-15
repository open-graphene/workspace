#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
REPORT_PATH="$CRATE_DIR/src/operation_model_skips.md"
SNAPSHOT_DIR="$(mktemp -d)"
trap 'rm -rf "$SNAPSHOT_DIR"' EXIT

cp "$CRATE_DIR/src/operation_variants.rs" "$SNAPSHOT_DIR/operation_variants.rs"
cp "$CRATE_DIR/src/operations.rs" "$SNAPSHOT_DIR/operations.rs"
cp "$REPORT_PATH" "$SNAPSHOT_DIR/operation_model_skips.md"
cp "$CRATE_DIR/src/wallet_api.rs" "$SNAPSHOT_DIR/wallet_api.rs"
cp -R "$CRATE_DIR/src/types" "$SNAPSHOT_DIR/types"

"$SCRIPT_DIR/gen.sh" --write

python3 - "$REPORT_PATH" <<'PY'
import sys
from pathlib import Path

REPORT_PATH = Path(sys.argv[1])
EXPECTED_UNSUPPORTED = set()


def strip_code(value: str) -> str:
    value = value.strip()
    if value.startswith("`") and value.endswith("`"):
        return value[1:-1]
    return value


def format_row(row):
    operation, tag, field, cpp_type, source, classification, reason = row
    return (
        f"operation={operation} tag={tag} field={field} "
        f"cpp_type={cpp_type} source={source} classification={classification} reason={reason}"
    )


def parse_report(path: Path):
    rows = []
    malformed = []
    in_table = False
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if line.startswith("| Operation | Tag | Field | Normalized C++ type |"):
            in_table = True
            continue
        if not in_table:
            continue
        if not line.startswith("|"):
            if rows:
                break
            continue
        if line.startswith("|---"):
            continue
        cells = [cell.strip() for cell in line.split("|")[1:-1]]
        if len(cells) != 7:
            malformed.append(f"line {line_number}: expected 7 table cells, found {len(cells)}: {line}")
            continue
        operation, tag, field, cpp_type, source, classification, reason = cells
        if not all((operation, tag, field, cpp_type, source, classification, reason)):
            malformed.append(f"line {line_number}: empty required table cell: {line}")
            continue
        rows.append((operation, tag, field, strip_code(cpp_type), source, classification, reason))
    return rows, malformed


rows, malformed = parse_report(REPORT_PATH)
if malformed:
    print("Malformed operation_model_skips.md table rows:", file=sys.stderr)
    for item in malformed:
        print(f"  - {item}", file=sys.stderr)
    sys.exit(1)

unsupported_rows = [row for row in rows if row[5] == "unsupported"]
rows_by_key = {}
for row in unsupported_rows:
    rows_by_key.setdefault((row[0], row[1], row[2], row[3]), []).append(row)
duplicate_items = sorted((key, matching) for key, matching in rows_by_key.items() if len(matching) > 1)
if duplicate_items:
    print("operation_model_skips.md contains duplicate unsupported row keys", file=sys.stderr)
    for key, matching in duplicate_items:
        print(
            f"Duplicate unsupported row key: operation={key[0]} tag={key[1]} field={key[2]} cpp_type={key[3]}",
            file=sys.stderr,
        )
        for row in matching:
            print(f"  - {format_row(row)}", file=sys.stderr)
    sys.exit(1)

actual = set(rows_by_key.keys())
expected = EXPECTED_UNSUPPORTED
unexpected = sorted(actual - expected)
missing = sorted(expected - actual)

if unexpected or missing:
    print("operation_model_skips.md unsupported boundary mismatch", file=sys.stderr)
    if unexpected:
        print("Unexpected unsupported rows:", file=sys.stderr)
        for key in unexpected:
            matching = [row for row in unsupported_rows if (row[0], row[1], row[2], row[3]) == key]
            if matching:
                for row in matching:
                    print(f"  - {format_row(row)}", file=sys.stderr)
            else:
                print(f"  - operation={key[0]} tag={key[1]} field={key[2]} cpp_type={key[3]}", file=sys.stderr)
    if missing:
        print("Missing expected unsupported rows:", file=sys.stderr)
        for operation, tag, field, cpp_type in missing:
            print(f"  - operation={operation} tag={tag} field={field} cpp_type={cpp_type}", file=sys.stderr)
    sys.exit(1)

print("operation_model_skips.md has no unsupported rows")
PY

diff -u "$SNAPSHOT_DIR/operation_variants.rs" "$CRATE_DIR/src/operation_variants.rs"
diff -u "$SNAPSHOT_DIR/operations.rs" "$CRATE_DIR/src/operations.rs"
diff -u "$SNAPSHOT_DIR/operation_model_skips.md" "$REPORT_PATH"
diff -u "$SNAPSHOT_DIR/wallet_api.rs" "$CRATE_DIR/src/wallet_api.rs"
diff -ru "$SNAPSHOT_DIR/types" "$CRATE_DIR/src/types"
