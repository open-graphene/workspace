# graphene-rs-v2

Clean-room workspace for the next Graphene Rust SDK/codegen architecture.

The current goal is to make the OpenRPC/Rust-types pipeline repeatable for any
Graphene-like blockchain before building higher-level SDK ergonomics.

## Current contents

```text
bin/gen.sh                      # one-command regeneration for all configured chains
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
crates/graphene-rpc/            # hand-written typed RPC runtime primitives
crates/graphene-chain-acta/
crates/graphene-chain-bitshares/
crates/graphene-chain-rsquared/
crates/graphene-chain-swaplock/
  src/generated/                # checked generated chain-local types
```

`graphene-codegen` is the pipeline owner. The OpenRPC directory still contains
Python/shell backend stages, but the orchestration entrypoint is Rust. `gen.sh`
is intentionally only a small project-local convenience wrapper.

`graphene-rpc` is the first hand-written SDK/runtime layer. It owns the shared
`OpenRpcParams` trait, `RpcTransport`, `RpcClient`, and `RpcError`; generated
chain crates implement that shared trait instead of defining their own copy.

## Generate all configured chains

From this directory:

```sh
bin/gen.sh
```

This regenerates BitShares, Acta, RSquared, and Swaplock, then runs:

```text
cargo fmt
cargo test
```

Each chain runs the same pipeline:

```text
C++ wallet/core headers
  -> OpenRPC spec
  -> typify schema
  -> Rust static variants
  -> Rust RPC params
  -> Rust schema types
```

The underlying Rust coordinator can also be run directly for one chain:

```sh
cargo run -p graphene-codegen --bin graphene-codegen -- generate chains/bitshares.toml
cargo run -p graphene-codegen --bin graphene-codegen -- generate chains/acta.toml
cargo run -p graphene-codegen --bin graphene-codegen -- generate chains/rsquared.toml
cargo run -p graphene-codegen --bin graphene-codegen -- generate chains/swaplock.toml
```

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
2. Add chain-specific roundtrip/fixture tests for BitShares, Acta, and RSquared.
3. Add a concrete HTTP or WebSocket `RpcTransport` implementation.
4. Port the Python Rust-facing backend stages into Rust modules incrementally.
