# OpenRPC codegen scripts

These scripts are the project-owned, generic form of the Swaplock POC pipeline.
They live under `crates/graphene-codegen` because they define generation policy
for Rust chain crates, even though several stages are still Python/shell while
we validate behavior.

## Boundary

This directory owns two related but separate phases:

1. **Upstream import:** C++ wallet/core metadata -> OpenRPC spec.
2. **Rust generation:** OpenRPC spec -> schema JSON, static variants, RPC params,
   and typify-generated Rust schema types.

The typify backend now lives in the same crate as the `openrpc-typify` binary:

```sh
cargo run -p graphene-codegen --bin openrpc-typify -- --help
```

Longer term, the rest of the Rust generation phase should move into Rust modules
inside `graphene-codegen`. The C++/Doxygen import phase may remain script-like
because it is an adapter to upstream source layout and external tools.

## Pipeline

Run all current stages from TOML:

```sh
./crates/graphene-codegen/openrpc/generate_from_config.py \
  --config examples/swaplock-openrpc.toml
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
  -> cargo run -p graphene-codegen --bin openrpc-typify
  -> Rust schema types

OpenRPC static_variant schemas
  -> gen_rust_variants.py
  -> Rust enums with Graphene [index, payload] serde

OpenRPC methods
  -> gen_rust_rpc.py
  -> Rust params structs with method constants and response types
```

## What is generic now

- Core root is a CLI/config argument.
- Chain name is a CLI/config argument.
- Wallet header can be overridden.
- Header roots are repeatable CLI/config arguments.
- OpenRPC spec path is a CLI/config argument.
- Rust output paths are CLI/config arguments.
- `generate_from_config.py` runs the stages from a TOML config and supports
  `--skip-openrpc` for reusing an existing spec during iteration.

## What is still POC-level

- `gen_wallet_spec.py` is still wallet-api oriented and looks for
  `graphene::wallet::wallet_api`.
- `gen_types_spec.py` is a pragmatic C++ text parser, not a full C++ parser.
- The Rust-facing generators other than the typify backend are still scripts;
  they should migrate into this crate once their behavior stabilizes.

## Tool dependencies

- `python3`
- `doxygen` for `gen_wallet_openrpc.sh`
- Rust/Cargo for the `openrpc-typify` binary in `graphene-codegen`
