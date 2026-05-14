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
    gen_openrpc.sh              # API header + C++ headers -> OpenRPC spec
    gen_api_spec.py             # Doxygen FC_API XML -> OpenRPC methods
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
use graphene_chain_swaplock::GetDynamicGlobalPropertiesParams;
use graphene_rpc::{HttpTransport, RpcClient};

let transport = HttpTransport::new("http://127.0.0.1:8090");
let client = RpcClient::new(transport);
let properties = client.call(GetDynamicGlobalPropertiesParams)?;
```

`HttpTransport` handles the JSON-RPC request/response envelope. Chain-specific
method names, positional params, and response types come from generated
`OpenRpcParams` implementations.

## Swaplock testnet sign + broadcast

The Swaplock testnet path now has a complete low-level transaction flow:

```text
transfer operation
  -> fee lookup through database_api
  -> transaction preparation from dynamic global properties
  -> WIF signing with Graphene-canonical compact signatures
  -> network_broadcast_api synchronous broadcast over WebSocket
```

Database/read RPC uses the public HTTP endpoint:

```text
https://node01.swaplock.chainpool.online:8090
```

Broadcast is not exposed as a direct HTTP method on that endpoint. It uses the
Graphene WebSocket `call(api_id, method, params)` protocol and discovers the
`network_broadcast` API id through the login API on the same connection:

```text
wss://node01.swaplock.chainpool.online:8090
```

Run the checked live broadcast fixture:

```sh
cargo test -p graphene-chain-swaplock \
  live_signs_and_broadcasts_tiny_transfer_with_wif \
  -- --ignored --nocapture
```

Run the executable example:

```sh
cargo run -p graphene-chain-swaplock --example swaplock_broadcast_transfer
```

Both send a tiny Swaplock testnet transfer from the shared test account to
`committee-account` and print only the transaction id, block number, and
transaction index. The example intentionally uses direct constants for the
shared testnet endpoint, accounts, and WIF so the happy path stays readable.
Those constants are exported by the crate as:

```rust
use graphene_chain_swaplock::{
    PUBLIC_SWAPLOCK_TESTNET_WIF, SWAPLOCK_TESTNET_FROM_ACCOUNT,
    SWAPLOCK_TESTNET_HTTP_URL, SWAPLOCK_TESTNET_TO_ACCOUNT,
    SWAPLOCK_TESTNET_WS_URL,
};
```

The reusable pieces are re-exported by the crate root:

```rust
use graphene_chain_swaplock::{
    broadcast_signed_transaction_synchronous_typed, build_transfer_operation,
    fetch_required_fee_for_transfer, lookup_exact_account_id, prepare_transaction,
    TransferDraft,
};
```

Internally these are split into small handwritten modules: `account.rs` for
account lookup helpers, `transfer.rs` for transfer/fee/transaction preparation,
`transaction.rs` for prepared/signed transaction and broadcast helpers, and
`testnet.rs` for the shared Swaplock testnet constants.

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
C++ app database_api/core headers
  -> OpenRPC spec
  -> typify schema
  -> Rust static variants
  -> Rust RPC params
  -> Rust schema types
  -> generated-output audit
```

The generated chain crates model the public node/database RPC surface. Wallet
RPC is intentionally not part of the core v2 SDK contract because wallet methods
can require local wallet state, keys, builder handles, or return shapes that do
not match public database nodes.

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
2. Expand ignored live HTTP smoke coverage with safe database API fixtures.
3. Port the Python Rust-facing backend stages into Rust modules incrementally.
