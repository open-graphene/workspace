#!/usr/bin/env python3
"""Run the generic Graphene OpenRPC generation pipeline from a TOML config.

The config intentionally describes paths, not a specific chain. Relative paths
are resolved from the config file's directory, except paths under `chain` that
point into the chain core: `wallet_header` and `header_roots` remain relative to
`chain.core_root` when they are not absolute.

Example:

    generate_from_config.py --config examples/swaplock-openrpc.toml

This script orchestrates the current POC stages:

1. wallet.hpp + C++ headers -> OpenRPC spec
2. OpenRPC components.schemas -> typify-compatible JSON Schema
3. OpenRPC static_variant schemas -> Rust variants.rs + variants.names
4. typify-compatible JSON Schema + variants.names -> Rust types.rs
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback not expected here.
    import tomli as tomllib  # type: ignore[no-redef]


SCRIPT_DIR = Path(__file__).resolve().parent


def resolve_from(base: Path, value: str | os.PathLike[str]) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (base / path).resolve()


def require_table(config: dict, name: str) -> dict:
    value = config.get(name)
    if not isinstance(value, dict):
        raise ConfigError(f"missing [{name}] table")
    return value


def require_str(table: dict, key: str, table_name: str) -> str:
    value = table.get(key)
    if not isinstance(value, str) or not value:
        raise ConfigError(f"missing required string: [{table_name}].{key}")
    return value


class ConfigError(Exception):
    pass


def run(command: list[str], dry_run: bool) -> None:
    printable = " ".join(str(part) for part in command)
    print(f"$ {printable}")
    if dry_run:
        return
    subprocess.run(command, check=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True, type=Path, help="pipeline TOML config")
    parser.add_argument("--dry-run", action="store_true", help="print commands without running them")
    parser.add_argument(
        "--skip-openrpc",
        action="store_true",
        help="reuse [openrpc].output and only run schema/variant extraction",
    )
    parser.add_argument(
        "--skip-rust",
        action="store_true",
        help="only generate/fill the OpenRPC spec",
    )
    parser.add_argument(
        "--skip-typify",
        action="store_true",
        help="generate schema/variants but do not run the typify Rust backend",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    config_path = args.config.resolve()
    config_dir = config_path.parent
    if not config_path.exists():
        raise ConfigError(f"config does not exist: {config_path}")

    with config_path.open("rb") as handle:
        config = tomllib.load(handle)

    chain = require_table(config, "chain")
    openrpc = require_table(config, "openrpc")
    rust = require_table(config, "rust")

    chain_name = require_str(chain, "name", "chain")
    core_root = resolve_from(config_dir, require_str(chain, "core_root", "chain"))
    spec_output = resolve_from(config_dir, require_str(openrpc, "output", "openrpc"))
    wallet_header = openrpc.get("wallet_header", "libraries/wallet/include/graphene/wallet/wallet.hpp")
    if not isinstance(wallet_header, str) or not wallet_header:
        raise ConfigError("[openrpc].wallet_header must be a non-empty string when present")

    header_roots = openrpc.get("header_roots", [])
    if header_roots is None:
        header_roots = []
    if not isinstance(header_roots, list) or not all(isinstance(root, str) for root in header_roots):
        raise ConfigError("[openrpc].header_roots must be an array of strings")

    schema_output = resolve_from(config_dir, require_str(rust, "schema_output", "rust"))
    variants_rs = resolve_from(config_dir, require_str(rust, "variants_rs", "rust"))
    variants_names = resolve_from(config_dir, require_str(rust, "variants_names", "rust"))
    types_rs = resolve_from(config_dir, require_str(rust, "types_rs", "rust"))
    types_preview = rust.get("types_preview")
    if types_preview is not None:
        if not isinstance(types_preview, str) or not types_preview:
            raise ConfigError("[rust].types_preview must be a non-empty string when present")
        types_preview = resolve_from(config_dir, types_preview)
    schema_title = rust.get("schema_title", f"{chain_name}_types_root")
    if not isinstance(schema_title, str) or not schema_title:
        raise ConfigError("[rust].schema_title must be a non-empty string when present")

    if not args.skip_openrpc:
        command = [
            str(SCRIPT_DIR / "gen_wallet_openrpc.sh"),
            "--chain-name",
            chain_name,
            "--core-root",
            str(core_root),
            "--wallet-header",
            wallet_header,
            "--output",
            str(spec_output),
        ]
        for root in header_roots:
            command.extend(["--header-root", root])
        run(command, args.dry_run)
    elif not spec_output.exists() and not args.dry_run:
        raise ConfigError(f"--skip-openrpc requested but spec does not exist: {spec_output}")

    if args.skip_rust:
        return 0

    run(
        [
            str(SCRIPT_DIR / "extract_typify_schema.py"),
            "--spec",
            str(spec_output),
            "--output",
            str(schema_output),
            "--title",
            schema_title,
        ],
        args.dry_run,
    )
    run(
        [
            str(SCRIPT_DIR / "gen_rust_variants.py"),
            "--spec",
            str(spec_output),
            "--out-rs",
            str(variants_rs),
            "--out-names",
            str(variants_names),
            "--source-label",
            str(spec_output),
        ],
        args.dry_run,
    )

    if args.skip_typify:
        return 0

    workspace_root = SCRIPT_DIR.parents[1]
    typify_command = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(workspace_root / "Cargo.toml"),
        "-p",
        "openrpc-typify",
        "--",
        "--schema",
        str(schema_output),
        "--out",
        str(types_rs),
        "--strip-names",
        str(variants_names),
    ]
    if types_preview is not None:
        typify_command.extend(["--preview", str(types_preview)])
    run(typify_command, args.dry_run)

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ConfigError as error:
        print(f"generate_from_config: {error}", file=sys.stderr)
        raise SystemExit(2)
    except subprocess.CalledProcessError as error:
        print(f"generate_from_config: command failed with exit code {error.returncode}", file=sys.stderr)
        raise SystemExit(error.returncode)
