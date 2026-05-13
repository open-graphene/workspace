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
  src/bin/graphene-codegen.rs   # Rust coordinator for generate/audit
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
`OpenRpcParams` trait, `RpcTransport`, `RpcClient`, `RpcError`, and a blocking
`HttpTransport`; generated chain crates implement the shared trait instead of
defining their own copy.

## Typed RPC usage shape

```rust
use graphene_chain_swaplock::AboutParams;
use graphene_rpc::{HttpTransport, RpcClient};

let transport = HttpTransport::new("http://127.0.0.1:8090");
let client = RpcClient::new(transport);
let about = client.call(AboutParams)?;
```

`HttpTransport` handles the JSON-RPC request/response envelope. Chain-specific
method names, positional params, and response types come from generated
`OpenRpcParams` implementations.

## Generate all configured chains

From this directory:

```sh
bin/gen.sh
```

This regenerates BitShares, Acta, RSquared, and Swaplock, audits each generated
binding set, then runs:

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
  -> generated-output audit
```

The underlying Rust coordinator can also be run directly for one chain:

```sh
cargo run -p graphene-codegen --bin graphene-codegen -- generate chains/bitshares.toml
cargo run -p graphene-codegen --bin graphene-codegen -- audit chains/bitshares.toml
```

Swap the config path for `chains/acta.toml`, `chains/rsquared.toml`, or
`chains/swaplock.toml` to target another chain.

## Audit checks

`graphene-codegen audit <config.toml>` compares the generated files with the
OpenRPC spec referenced by that config. It checks:

- every schema has a generated Rust type or static-variant enum,
- every method has a generated params struct and `OpenRpcParams` impl,
- every static-variant schema has an enum and `variants.names` entry,
- generated `rpc.rs` imports `graphene_rpc::OpenRpcParams` and does not define a
  local copy,
- TODO placeholders and `serde_json::Value` RPC fallbacks are reported.

TODO placeholders are warnings for now because Acta and RSquared still expose
known unresolved C++ reflection debt.

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

1. Add chain-specific roundtrip/fixture tests for BitShares, Acta, and RSquared.
2. Add a live/ignored HTTP smoke test or example against a local wallet/node.
3. Port the Python Rust-facing backend stages into Rust modules incrementally.
