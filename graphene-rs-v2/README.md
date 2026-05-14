# graphene-rs-v2

Clean-room workspace for the next Graphene Rust SDK/codegen architecture.

The current goal is to make the OpenRPC/Rust-types pipeline repeatable for any
Graphene-like blockchain before building higher-level SDK ergonomics.

## Current contents

```text
bin/gen.sh                      # one-command regeneration for all configured chains
chains/                         # per-chain generation configs
  acta.toml                     # may contain multiple API surfaces
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
crates/graphene-transaction/    # shared transaction parsing/preparation primitives
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
`OpenRpcParams` and `OpenRpcCallbackParams` traits, `RpcTransport`, `RpcClient`,
`RpcError`, blocking `HttpTransport`, and the session-oriented WebSocket runtime.
The runtime code is split into focused modules (`database_callbacks`, `error`,
`params`, `scalar`, `http`, `jsonrpc`, and `ws`) while `lib.rs` keeps the public
re-export surface stable. The `ws` module is further split between
session-facing APIs, dispatcher internals, shared protocol helpers, public
handle types, and an explicit one-shot WebSocket transport for simple
one-call-per-socket flows. Generated chain crates implement the shared traits
instead of defining their own copies. Reusable database callback params such as
`set_block_applied_callback` live in `database_callbacks`; chain crates re-export
that shared helper alongside the shared callback payload types for convenience.

`graphene-transaction` contains chain-independent transaction primitives that
were proven across Swaplock and Acta first: exact account lookup result
validation, reference-block prefix parsing from Graphene block ids, transaction
header field derivation from dynamic global property values, typed synchronous
broadcast result parsing, and shared signing over Graphene-encoded transaction
bytes. Chain crates still keep generated RPC params and generated
transaction/operation/signed-transaction types chain-local.

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

`graphene-rpc` now exposes a session-oriented WebSocket runtime for that model:

```rust
use graphene_rpc::{GrapheneWebSocketSession, RpcClient};
use std::sync::Arc;

let session = Arc::new(GrapheneWebSocketSession::connect(SWAPLOCK_TESTNET_WS_URL)?);
let broadcast_api = session.login_api("network_broadcast")?;
let broadcast_client = RpcClient::new(session.api_transport(&broadcast_api));
```

The session keeps Graphene API ids scoped to their owning socket. A background
WebSocket dispatcher owns all socket reads and writes: generated RPC responses
are routed by JSON-RPC request id, and Graphene server pushes are routed by
callback id. Ordinary generated request/response methods implement
`OpenRpcParams`; callback-aware methods implement the separate
`OpenRpcCallbackParams` trait so the WebSocket session can allocate and prepend
the local callback id before method-specific params. The dispatcher fails
pending requests and clears callback routing on disconnect; reconnect/resubscribe
policy is intentionally out of scope for this step. The chain crates expose a
first callback proof helper for signed transactions:

```rust
use graphene_chain_swaplock::broadcast_signed_transaction_with_callback_typed;

let result = broadcast_signed_transaction_with_callback_typed(
    &session,
    &broadcast_api,
    signed,
)?;
```

Run the checked live broadcast fixtures:

```sh
cargo test -p graphene-chain-swaplock \
  live_signs_and_broadcasts_tiny_transfer_with_wif \
  -- --ignored --nocapture

cargo test -p graphene-chain-swaplock \
  live_broadcasts_tiny_transfer_with_callback \
  -- --ignored --nocapture

cargo test -p graphene-chain-swaplock \
  live_receives_block_applied_callback_notice \
  -- --ignored --nocapture
```

The callback proof has been verified against the Swaplock testnet; the real
callback notice payload arrives as a single-argument array containing the
`transaction_confirmation` object, while `broadcast_transaction_synchronous`
returns that object directly. `broadcast_transaction_with_callback` is kept as a
handwritten helper for now, not a generated `OpenRpcParams` method, because the
first wire parameter is a local callback id allocated by the WebSocket session
rather than user-supplied RPC input. Internally it now uses
`OpenRpcCallbackParams` and the typed `call_with_callback_once(...)` runtime path
instead of ad-hoc raw parameter assembly. `set_block_applied_callback` has also
been verified live as the first persistent callback proof: it registers a
callback on the database API and receives a later block id through
`method: "notice"`.

Run the executable examples:

```sh
cargo run -p graphene-chain-swaplock --example swaplock_broadcast_transfer
cargo run -p graphene-chain-swaplock --example swaplock_block_applied_callback
cargo run -p graphene-chain-swaplock --example swaplock_dynamic_global_properties_subscription
```

`swaplock_block_applied_callback` is a non-mutating WebSocket subscription
example: it uses the typed handwritten `set_block_applied_callback` helper,
prints the next block id received through the dispatcher, unsubscribes locally,
and exits. `swaplock_dynamic_global_properties_subscription` uses
`set_subscribe_callback` plus `get_objects(["2.1.0"], true)` to print the
initial dynamic global properties object and the next typed update notice.

The broadcast example and live broadcast tests send a tiny Swaplock testnet transfer from the shared test account to
`committee-account` and print only the transaction id, block number, and
transaction index. The example intentionally uses direct constants for the shared testnet endpoint,
accounts, transfer asset, transfer amount, and WIF so the happy path stays
readable. Those constants are exported by the crate as:

```rust
use graphene_chain_swaplock::{
    PUBLIC_SWAPLOCK_TESTNET_WIF, SWAPLOCK_TESTNET_FROM_ACCOUNT,
    SWAPLOCK_TESTNET_HTTP_URL, SWAPLOCK_TESTNET_TO_ACCOUNT,
    SWAPLOCK_TESTNET_TRANSFER_AMOUNT, SWAPLOCK_TESTNET_TRANSFER_ASSET,
    SWAPLOCK_TESTNET_WS_URL,
};
```

The reusable pieces are re-exported by the crate root:

```rust
use graphene_chain_swaplock::{
    broadcast_signed_transaction_synchronous_typed, lookup_exact_account_id,
    prepare_transfer_transaction, set_block_applied_callback,
    subscribe_dynamic_global_properties, TransferDraft,
};
```

A transfer remains explicit and low-level. The helper prepares the transaction;
the caller still chooses the accounts, fee asset, signer, and broadcast path:

```rust
let from = lookup_exact_account_id(&database_client, SWAPLOCK_TESTNET_FROM_ACCOUNT)?;
let to = lookup_exact_account_id(&database_client, SWAPLOCK_TESTNET_TO_ACCOUNT)?;
let chain_id = database_client.call(GetChainIdParams)?;
let chain_id = ChainId::try_from(chain_id.as_str())?;

let prepared = prepare_transfer_transaction(
    &database_client,
    TransferDraft {
        from,
        to,
        amount: SWAPLOCK_TESTNET_TRANSFER_AMOUNT,
        asset_id: SWAPLOCK_TESTNET_TRANSFER_ASSET.to_owned(),
    },
    SWAPLOCK_TESTNET_TRANSFER_ASSET,
    chrono::Duration::minutes(5),
)?;

let signed = prepared.sign(&chain_id, &signer)?;
let response = broadcast_signed_transaction_synchronous_typed(&broadcast_client, signed)?;
```

Internally these are split into small handwritten modules: `account.rs` for
account lookup helpers, `transfer.rs` for transfer/fee/transaction preparation
including the `prepare_transfer_transaction` builder, `transaction.rs` for
prepared/signed transaction and broadcast helpers, and `testnet.rs` for the
shared Swaplock testnet constants.

## Acta transfer/broadcast adapter

Acta now has the same low-level handwritten adapter shape as Swaplock, with
chain-local generated types preserved at the boundary:

```rust
use graphene_chain_acta::{
    lookup_exact_account_id, prepare_transfer_transaction, set_block_applied_callback,
    subscribe_dynamic_global_properties, TransferDraft,
};
```

The Acta crate includes `account.rs`, `transfer.rs`, `transaction.rs`, and a
transfer-only binary `codec.rs`. The chain config contains a separate
`network_broadcast_api` surface that generates `graphene_chain_acta::broadcast`,
mirroring Swaplock's split between public `database_api` reads and broadcast
RPC while preserving one config file per chain.

Run the checked live broadcast and non-mutating callback fixtures:

```sh
cargo test -p graphene-chain-acta \
  live_signs_and_broadcasts_tiny_transfer_with_wif \
  -- --ignored --nocapture

cargo test -p graphene-chain-acta \
  live_broadcasts_tiny_transfer_with_callback \
  -- --ignored --nocapture

cargo test -p graphene-chain-acta \
  live_receives_block_applied_callback_notice \
  -- --ignored --nocapture
```

The Acta broadcast callback proof has also been verified against its testnet,
giving the WebSocket callback ABI two live-proven broadcast-callback chains:
Swaplock and Acta. The non-mutating Acta `set_block_applied_callback` proof has
also been verified live: it registered a database callback and received a real
block id through the shared dispatcher-backed callback path.

Run the executable examples:

```sh
cargo run -p graphene-chain-acta --example acta_broadcast_transfer
cargo run -p graphene-chain-acta --example acta_dynamic_global_properties_subscription
```

Both broadcast examples send a tiny Acta testnet transfer from the shared test account to
`committee-account` and print only the transaction id, block number, and
transaction index. The dynamic global properties example is non-mutating: it
subscribes over WebSocket, prints the initial `2.1.0` object, waits for one
update notice, unsubscribes locally, and exits. The testnet constants are
exported by the crate as:

```rust
use graphene_chain_acta::{
    ACTA_TESTNET_FROM_ACCOUNT, ACTA_TESTNET_HTTP_URL,
    ACTA_TESTNET_TO_ACCOUNT, ACTA_TESTNET_TRANSFER_AMOUNT,
    ACTA_TESTNET_TRANSFER_ASSET, ACTA_TESTNET_WS_URL,
    PUBLIC_ACTA_TESTNET_WIF,
};
```

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

Each chain config may contain one or more `[[surfaces]]`; Acta and Swaplock use
that to generate both their public `database_api` bindings and separate
`network_broadcast_api` bindings from one chain config. Each surface runs the
same pipeline:

```text
C++ app API/core headers
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
`chains/swaplock.toml` to target another chain. When a config has multiple
surfaces, the command generates/audits all of them in file order.

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
