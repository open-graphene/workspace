# graphene-rs-v2

Clean-room workspace for the next Graphene Rust SDK/codegen architecture.

The current goal is to make the OpenRPC/Rust-types pipeline repeatable for any
Graphene-like blockchain before building higher-level SDK ergonomics.

## Current contents

```text
bin/gen.sh                      # one-command Swaplock regeneration entrypoint
chains/                         # per-chain generation configs
  acta.toml
  bitshares.toml
  rsquared.toml
  swaplock.toml
crates/graphene-codegen/
  src/bin/graphene-codegen.rs   # Rust coordinator for the generation pipeline
  src/bin/openrpc-typify.rs     # typify backend: schema.json -> types.rs
  openrpc/                      # script backends spawned by graphene-codegen
    gen_wallet_openrpc.sh       # wallet.hpp + C++ headers -> OpenRPC spec
    gen_wallet_spec.py          # Doxygen wallet_api XML -> OpenRPC methods
    gen_types_spec.py           # FC_REFLECT/static_variant scan -> OpenRPC schemas
    extract_typify_schema.py    # OpenRPC schemas -> JSON Schema $defs for typify
    gen_rust_variants.py        # static_variant schemas -> Rust [tag, payload] enums
    gen_rust_rpc.py             # OpenRPC methods -> Rust params/response bindings
crates/graphene-chain-swaplock/
  src/generated/                # checked generated Swaplock chain-local types
```

`graphene-codegen` is the pipeline owner. The OpenRPC directory still contains
Python/shell backend stages, but the orchestration entrypoint is Rust. `gen.sh`
is intentionally only a small project-local convenience wrapper.

## Generate Swaplock

From this directory:

```sh
bin/gen.sh
```

This runs the full path:

```text
Swaplock C++ wallet/core headers
  -> OpenRPC spec
  -> typify schema
  -> Rust static variants
  -> Rust RPC params
  -> Rust schema types
  -> cargo fmt
  -> cargo test
```

The underlying Rust coordinator can also be run directly:

```sh
cargo run -p graphene-codegen --bin graphene-codegen -- \
  generate chains/swaplock.toml
```

Other chain configs are available under `chains/`:

```sh
cargo run -p graphene-codegen --bin graphene-codegen -- generate chains/bitshares.toml
cargo run -p graphene-codegen --bin graphene-codegen -- generate chains/acta.toml
cargo run -p graphene-codegen --bin graphene-codegen -- generate chains/rsquared.toml
```

Those currently write generated Rust files to `target/generated/<chain>/` until
matching chain crates are added.

## Individual backend stages

The scripts under `crates/graphene-codegen/openrpc/` are backend stages. They are
useful for debugging one phase, but normal generation should go through
`graphene-codegen` or `bin/gen.sh`.

Run the typify backend directly:

```sh
cargo run -p graphene-codegen --bin openrpc-typify -- \
  --schema /tmp/swaplock-schema.json \
  --out /tmp/types.rs \
  --strip-names /tmp/variants.names
```

## Next steps

1. Add a repeatable generated-output audit command under `graphene-codegen`.
2. Port the Python Rust-facing backend stages into Rust modules incrementally.
3. Add generated chain crates for BitShares, Acta, and RSquared.
4. Build the typed RPC SDK only after the codegen ownership boundary is stable.
