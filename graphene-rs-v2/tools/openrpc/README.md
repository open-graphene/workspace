# OpenRPC codegen scripts

These tools are the project-owned, generic form of the Swaplock POC scripts.
They are intentionally path-parameterized so the same pipeline can target any
Graphene-like chain core with a wallet API and FC reflection metadata.

## Pipeline

Run all current stages from TOML:

```sh
./tools/openrpc/generate_from_config.py --config examples/swaplock-openrpc.toml
```

Individual stages:

```text
wallet.hpp + Doxygen
  -> gen_wallet_spec.py
  -> OpenRPC methods with placeholder schemas

C++ header roots
  -> gen_types_spec.py
  -> filled OpenRPC components.schemas

OpenRPC components.schemas
  -> extract_typify_schema.py
  -> JSON Schema $defs document for typify
  -> openrpc-typify
  -> Rust schema types

OpenRPC static_variant schemas
  -> gen_rust_variants.py
  -> Rust enums with Graphene [index, payload] serde
```

## What is generic now

- Core root is a CLI argument.
- Chain name is a CLI argument.
- Wallet header can be overridden.
- Header roots are repeatable CLI arguments.
- OpenRPC spec path is a CLI argument.
- Rust variants output paths are CLI arguments.
- Rust typify output path is a CLI argument.
- `generate_from_config.py` runs the stages from a TOML config and supports
  `--skip-openrpc` for reusing an existing spec during iteration.

## What is still POC-level

- `gen_wallet_spec.py` is still wallet-api oriented and looks for
  `graphene::wallet::wallet_api`.
- `gen_types_spec.py` is a pragmatic C++ text parser, not a full C++ parser.
- Typify is currently invoked through the small Rust `openrpc-typify` runner;
  the next step is integrating that runner into a proper `graphene-codegen` crate.
- No chain crate is generated yet in `graphene-rs-v2`.

## Tool dependencies

- `python3`
- `doxygen` for `gen_wallet_openrpc.sh`
- Rust/Cargo for `openrpc-typify`
