# OpenRPC codegen backend stages

These scripts are backend stages for the `graphene-codegen` Rust coordinator.
They are the project-owned, generic form of the Swaplock POC pipeline.

Normal generation should use the Rust coordinator, not these scripts directly:

```sh
cargo run -p graphene-codegen --bin graphene-codegen -- \
  generate chains/swaplock.toml
```

Or the project-local shortcut:

```sh
bin/gen.sh
```

## Boundary

This directory contains process backends for two related phases:

1. **Upstream import:** C++ wallet/core metadata -> OpenRPC spec.
2. **Rust generation helpers:** OpenRPC spec -> schema JSON, static variants,
   and RPC params.

The typify backend lives in the same crate as the `openrpc-typify` binary:

```sh
cargo run -p graphene-codegen --bin openrpc-typify -- --help
```

Longer term, the Rust generation helpers should move into Rust modules inside
`graphene-codegen`. The C++/Doxygen import phase may remain script-like because
it adapts to upstream source layout and external tools.

## Pipeline

The Rust coordinator runs these stages:

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
  -> cargo run -p graphene-codegen --bin openrpc-typify
  -> Rust schema types

OpenRPC static_variant schemas
  -> gen_rust_variants.py
  -> Rust enums with Graphene [index, payload] serde

OpenRPC methods
  -> gen_rust_rpc.py
  -> Rust params structs with method constants and response types
```

## What is still POC-level

- `gen_wallet_spec.py` is still wallet-api oriented and looks for
  `graphene::wallet::wallet_api`.
- `gen_types_spec.py` is a pragmatic C++ text parser, not a full C++ parser.
- `gen_rust_variants.py`, `gen_rust_rpc.py`, and `extract_typify_schema.py` are
  still Python backends spawned by the Rust coordinator.

## Tool dependencies

- `python3`
- `doxygen` for `gen_wallet_openrpc.sh`
- Rust/Cargo for the `openrpc-typify` binary in `graphene-codegen`
