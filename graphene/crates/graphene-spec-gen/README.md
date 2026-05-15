# graphene-spec-gen

`graphene-spec-gen` is the OpenGraphene contract compiler for the Graphene SDK workspace. It gives future SDK and emitter work one stable entrypoint for validating a chain contract, inspecting the lowered intermediate representation (IR), generating reviewable TypeScript and Dart metadata samples, and emitting byte-level conformance fixtures.

The crate is intentionally small and diagnostic-first. When a command fails, the CLI error text is the durable triage surface: validation errors include contract paths, generation errors name the unsupported target or argument, and conformance fixture errors identify the fixture phase that drifted.

## Purpose

OpenGraphene is a chain contract document that describes enough Graphene RPC, codec, transaction, operation, and callback semantics for downstream SDK/code-generator work without requiring contributors to reverse-engineer Rust structs or handwritten chain helpers.

The current milestone supports this contract-first developer loop:

1. Validate the mandatory Swaplock and Acta contract JSON documents.
2. Emit the JSON Schema for authoring and review tooling.
3. Lower each contract, enriched by its matching OpenRPC metadata, into JSON IR.
4. Generate TypeScript and Dart descriptor samples from the IR as prototype metadata drift checks.
5. Emit conformance fixtures from the Rust reference codec and signer.
6. Run the workspace tests that lock those contracts in place.

This milestone deliberately stops at contract JSON, IR, generated metadata samples, conformance fixtures, and tests. Production TypeScript or Dart SDK packaging is an active non-goal until a later milestone adds runtime codecs, signing helpers, transports, and release criteria.

## Architecture

The generator pipeline is deliberately staged:

```text
OpenGraphene JSON contract
        |
        v
model + validation  ----> path-specific contract diagnostics
        |
        v
lowering (+ optional OpenRPC enrichment)
        |
        v
IR JSON  ----> inspect-ir for humans and future emitters
        |
        +----> TypeScript metadata emitter
        +----> Dart metadata emitter
        +----> Rust reference conformance fixture emitter
```

Important source areas:

- `src/model.rs` defines the versioned contract model serialized by fixtures.
- `src/validation.rs` enforces semantic references such as API, type, operation, transaction, and callback links.
- `src/openrpc.rs` parses the small OpenRPC subset used for method enrichment.
- `src/lower.rs` and `src/ir.rs` produce the emitter-facing IR.
- `src/emit/typescript.rs` and `src/emit/dart.rs` write generated metadata samples.
- `src/conformance.rs` builds byte-level reference fixtures for operation encoding, transaction encoding, digest construction, signing, and broadcast JSON envelopes.
- `src/bin/graphene-spec-gen.rs` is the CLI entrypoint and should remain the documented operator surface.

## Contract file layout

The canonical compact examples cover the two mandatory M001 chains:

```text
crates/graphene-spec-gen/fixtures/swaplock.opengraphene.json
crates/graphene-spec-gen/fixtures/acta.opengraphene.json
```

They may be paired with OpenRPC metadata for method descriptions, parameters, and results:

```text
crates/graphene-spec-gen/fixtures/swaplock.openrpc.json
crates/graphene-spec-gen/fixtures/acta.openrpc.json
```

A v0.1 OpenGraphene document contains these root sections:

- `openGraphene`: contract version string. Current fixtures use `"0.1"`.
- `openrpc`: optional path hint to the matching OpenRPC description.
- `chain`: chain metadata including name, chain id, and endpoints.
- `apis`: named Graphene API surfaces and their wire `grapheneName`.
- `methodBindings`: mapping from OpenRPC method names to declared Graphene APIs.
- `codec`: named binary codec types used by RPCs, operations, transactions, and callbacks.
- `operations`: static-variant operation sets with Graphene operation ids and payload types.
- `transaction`: signed transaction type, operation variant set, digest rules, and signature rules.
- `callbacks`: callback-taking RPC contracts where the SDK allocates session-local callback ids.
- `shapeClassifications`: optional validation evidence for intentionally raw or unsupported contract shapes.

### `apis`

`apis` declares logical API handles and the Graphene API name used on the wire. `access` is currently either `default` or `login_api`.

```json
{
  "apis": {
    "database": {
      "grapheneName": "database",
      "access": "default"
    },
    "network_broadcast": {
      "grapheneName": "network_broadcast",
      "access": "login_api"
    }
  }
}
```

Validation requires every `grapheneName` to be non-empty.

### `methodBindings`

`methodBindings` assigns each OpenRPC method to one declared API. This keeps the OpenRPC method shape separate from Graphene API routing.

```json
{
  "methodBindings": {
    "get_dynamic_global_properties": {
      "api": "database"
    },
    "broadcast_transaction": {
      "api": "network_broadcast"
    }
  }
}
```

Validation rejects bindings that reference an unknown `apis` key and reports the failing path, for example `methodBindings.broadcast_transaction.api`.

### `codec`

`codec.types` declares named types used by transactions, operation payloads, method params/results, and callback notices. v0.1 currently supports `struct` types with fields. Field type references may be:

- a builtin or named type: `"int64"`, `"Asset"`
- an array: `{ "array": "Operation" }`
- an optional value: `{ "optional": "bytes" }`

```json
{
  "codec": {
    "types": {
      "Asset": {
        "kind": "struct",
        "fields": [
          { "name": "amount", "type": "int64" },
          { "name": "asset_id", "type": "object_id" }
        ]
      },
      "TransferOperation": {
        "kind": "struct",
        "fields": [
          { "name": "fee", "type": "Asset" },
          { "name": "memo", "type": { "optional": "bytes" } }
        ]
      }
    }
  }
}
```

Builtins currently include `bool`, `bytes`, `uint8`, `uint16`, `uint32`, `uint64`, `int64`, `object_id`, `public_key`, `signature`, `string`, `time_point_sec`, and `void`. Validation rejects duplicate fields and unknown type references.

### `operations`

`operations` declares Graphene static-variant sets. Each variant has a numeric operation id, a stable operation name, and a codec payload type.

```json
{
  "operations": {
    "Operation": [
      {
        "id": 0,
        "name": "transfer",
        "type": "TransferOperation"
      }
    ]
  }
}
```

Variant ids and names must be unique inside their variant set. Variant payload types must resolve to known codec types.

### `transaction`

`transaction` describes the signed transaction type, the operation variant set embedded in it, and digest/signature rules.

```json
{
  "transaction": {
    "type": "SignedTransaction",
    "operationVariant": "Operation",
    "digest": {
      "algorithm": "sha256",
      "preimage": ["chain_id", "serialized_transaction"]
    },
    "signature": {
      "curve": "secp256k1",
      "format": "graphene_compact_recoverable",
      "canonical": true
    }
  }
}
```

Validation requires `type` to resolve to a known codec type, `operationVariant` to reference a declared operation variant set, and `digest.preimage` to contain at least one preimage part.

### `callbacks`

`callbacks` describes RPC methods where the client allocates a callback id and receives an asynchronous payload later. The callback parameter is modeled separately from ordinary request params so SDKs can hide session-local callback id allocation from end users.

```json
{
  "callbacks": {
    "broadcast_transaction_with_callback": {
      "api": "network_broadcast",
      "requestParams": ["SignedTransaction"],
      "directResult": "void",
      "callbackParam": {
        "position": 0,
        "allocatedBy": "client"
      },
      "callbackPayload": "BroadcastResult",
      "callbackLifetime": "once"
    },
    "set_block_applied_callback": {
      "api": "database",
      "callbackParam": {
        "position": 0,
        "allocatedBy": "client"
      },
      "callbackPayload": "BlockAppliedNotice",
      "callbackLifetime": "persistent"
    }
  }
}
```

`callbackLifetime` is `once` for one-shot notices and `persistent` for subscriptions. Validation checks callback API references, request parameter types, direct result types, and callback payload types.

### `shapeClassifications`

`shapeClassifications` is optional validation evidence for Graphene shapes that need explicit triage instead of being silently hidden in generator behavior. Each entry names a contract `path`, a `classification`, and a human-readable `reason`.

```json
{
  "shapeClassifications": [
    {
      "path": "codec.types.TransferOperation.fields[4].type",
      "classification": "approved_raw_fallback",
      "reason": "memo bytes are intentionally retained as raw encrypted payload bytes"
    }
  ]
}
```

Supported classifications are:

- `approved_raw_fallback`: accepted by validation, surfaced as a deterministic warning in the validation report/IR diagnostics, and printed by `graphene-spec-gen validate` without making the command fail. Use this when raw bytes or another already-modeled fallback is intentional and reviewed.
- `unsupported_shape`: rejected by validation. The error reports the classified contract path and includes `unsupported_shape` in the message so future agents can distinguish an intentionally blocked Graphene shape from ordinary reference drift.

Validation requires both `path` and `reason` to be non-empty. These diagnostics are path-only authoring metadata; they must not include secrets, live endpoint output, or runtime logs.

## Quickstart

Run these commands from the `graphene/` workspace root.

```sh
# 1. Regenerate published OpenGraphene specs for mandatory chains.
bin/gen.sh

# 2. Validate the canonical OpenGraphene contracts.
cargo run -p graphene-spec-gen -- validate specs/swaplock.opengraphene.json
cargo run -p graphene-spec-gen -- validate specs/acta.opengraphene.json

# 3. Print the JSON Schema for the contract model.
cargo run -p graphene-spec-gen -- schema

# 4. Inspect lowered IR enriched by OpenRPC metadata.
cargo run -p graphene-spec-gen -- inspect-ir \
  crates/graphene-spec-gen/fixtures/swaplock.opengraphene.json \
  --openrpc crates/graphene-spec-gen/fixtures/swaplock.openrpc.json
cargo run -p graphene-spec-gen -- inspect-ir \
  crates/graphene-spec-gen/fixtures/acta.opengraphene.json \
  --openrpc crates/graphene-spec-gen/fixtures/acta.openrpc.json

# 5. Generate prototype TypeScript and Dart metadata samples into a scratch directory.
#    These outputs are review/drift evidence, not production SDK packages.
rm -rf /tmp/graphene-spec-gen-samples
cargo run -p graphene-spec-gen -- generate \
  specs/swaplock.opengraphene.json \
  --openrpc crates/graphene-spec-gen/fixtures/swaplock.openrpc.json \
  --target all \
  --out /tmp/graphene-spec-gen-samples

# 6. Emit conformance fixtures into a scratch directory.
rm -rf /tmp/graphene-spec-gen-conformance
cargo run -p graphene-spec-gen -- conformance-fixtures \
  --out /tmp/graphene-spec-gen-conformance

# 7. Run the crate test suite.
cargo test -p graphene-spec-gen
```

Expected success signals:

- `opengraphene-specs` prints `generated OpenGraphene specs into <dir>` and writes `acta.opengraphene.json` plus `swaplock.opengraphene.json`.
- `validate` prints `<path>: valid OpenGraphene contract` for both Swaplock and Acta fixtures.
- `schema` prints JSON with title `OpenGrapheneDocument`.
- `inspect-ir` prints JSON containing `methods`, `operations`, `callbacks`, and `transaction` metadata.
- `generate --target all` prints `generated target 'all' into <dir>` and writes TypeScript, Dart, and Rust outputs.
- `conformance-fixtures` prints `generated conformance fixtures into <dir>` and writes `transfer.json`.

The installed or debug binary uses the same subcommands:

```sh
graphene-spec-gen validate <path>
graphene-spec-gen schema
graphene-spec-gen inspect-ir <opengraphene-path> [--openrpc <openrpc-path>]
graphene-spec-gen generate <opengraphene-path> [--openrpc <openrpc-path>] --target <typescript|dart|rust|all> --out <dir>
graphene-spec-gen conformance-fixtures --out <dir>
graphene-spec-gen opengraphene-specs --out <dir>
```

## Published OpenGraphene specs

`bin/gen.sh` regenerates the checked-in mandatory chain specs under `specs/`:

```text
specs/acta.opengraphene.json
specs/swaplock.opengraphene.json
```

The script runs `cargo run -p graphene-spec-gen -- opengraphene-specs --out <tmp-dir>`, then atomically replaces `specs/`. The CLI validates its built-in Acta and Swaplock OpenGraphene documents before writing them, so a broken embedded contract fails generation instead of publishing stale specs.

```sh
bin/gen.sh
bin/gen_ts.sh
bin/gen_dart.sh
bin/gen_rs.sh
```

`bin/gen_ts.sh` regenerates the current TypeScript generated artifacts into `../graphene-ts/graphene-bindings-swaplock/src/index.ts` and `../graphene-ts/graphene-bindings-acta/src/index.ts`. `bin/gen_dart.sh` regenerates the current Dart generated artifacts into `../graphene-dart/graphene-bindings-swaplock/lib/open_graphene.dart` and `../graphene-dart/graphene-bindings-acta/lib/open_graphene.dart`. `bin/gen_rs.sh` regenerates the current Rust generated artifacts into `../graphene-rs/graphene-bindings-swaplock/src/lib.rs` and `../graphene-rs/graphene-bindings-acta/src/lib.rs`, with standalone `Cargo.toml` files for `cargo check`. These wrapper outputs use the existing prototype emitters and are not yet full production SDK runtimes; they are stable generated SDK artifacts for the mandatory Swaplock and Acta specs.

## Emitted targets

`generate` currently supports four target selectors:

- `--target typescript`
- `--target dart`
- `--target rust`
- `--target all`

These targets are prototype metadata samples only. They are useful for checking that validated OpenGraphene/OpenRPC contracts lower into stable language-facing descriptors, but they are not production SDK packages and do not include transaction encoders, signers, transports, packaging, release automation, or runtime compatibility promises.

For `--target all`, the generated file layout is:

```text
<out>/typescript/index.ts
<out>/dart/lib/open_graphene.dart
<out>/rust/src/lib.rs
```

The checked-in generated samples live at:

```text
crates/graphene-spec-gen/fixtures/generated-samples/typescript/index.ts
crates/graphene-spec-gen/fixtures/generated-samples/dart/lib/open_graphene.dart
```

The generated samples are metadata and type descriptors, not full runtime SDKs. They currently expose:

- codec structs/classes for contract types such as `Asset`, `SignedTransaction`, and `TransferOperation`;
- operation variant metadata, including the `transfer` operation id;
- RPC method descriptors with API routing, params, result metadata, and callback links;
- callback metadata with payload type, lifetime, and callback parameter position;
- transaction digest/signature metadata.

`tests/generated_samples.rs` regenerates both targets into a temporary directory and compares them byte-for-byte with the checked-in samples. If emitter behavior changes intentionally, regenerate the samples and review the diff as part of that change.

## Conformance fixtures

Conformance fixtures are stable JSON files generated from the Rust reference codec and signer. They are intended for non-Rust native implementations, currently TypeScript and Dart, to prove byte-for-byte compatibility before those SDKs own their own transaction codec and signing code.

Generate the current fixture with:

```sh
cargo run -p graphene-spec-gen -- conformance-fixtures --out crates/graphene-spec-gen/fixtures/conformance
```

The canonical fixture is:

```text
crates/graphene-spec-gen/fixtures/conformance/transfer.json
```

Each fixture has this contract:

- `schemaVersion`: fixture schema version. Consumers should reject unknown major versions instead of silently accepting changed semantics.
- `name`: stable test case name, for example `swaplock-transfer`.
- `chainId`: chain id used in the signing digest preimage.
- `input.operationName`: operation variant under test.
- `input.operationJson`: JSON-ish operation payload that a generated SDK should accept before binary encoding.
- `input.transactionJson`: unsigned transaction JSON that wraps the operation and supplies header fields such as ref block and expiration.
- `input.privateKeyWif`: deterministic fixture key used only for conformance tests.
- `expected.operationHex`: expected Graphene operation bytes, including the static-variant operation id and encoded payload.
- `expected.transactionHex`: expected unsigned transaction hex.
- `expected.digestHex`: expected signing digest over `chainId || transaction bytes`.
- `expected.publicKey`: public key derived from `input.privateKeyWif`.
- `expected.signatureHex`: expected compact recoverable Graphene signature bytes for the digest.
- `expected.signatureMetadata`: digest algorithm, curve, signature format, and canonical-signature requirement that TypeScript and Dart signers must enforce.
- `expected.broadcastPayload`: JSON-RPC broadcast method, API name, and signed transaction JSON envelope expected after the signature is attached.

Consumer tests should compare each expected field independently in this order: operation hex, transaction hex, digest hex, public key, signature hex, then broadcast payload. Keeping those assertions separate makes byte-level drift actionable: an operation mismatch points at operation variant or field encoding, a transaction mismatch points at transaction framing, a digest mismatch points at preimage construction, a signature mismatch points at key/signing behavior, and a broadcast-payload mismatch points at JSON envelope assembly.

## Diagnostics and failure triage

Use the CLI command that maps to the failing layer:

- Contract authoring failures: run `validate` and inspect the reported contract path. Approved raw fallbacks are printed as warnings; unsupported-shape classifications fail validation with the classified path and `unsupported_shape` marker.
- Schema/tooling drift: run `schema` and compare the `OpenGrapheneDocument` schema.
- OpenRPC enrichment or emitter input drift: run `inspect-ir` with the same `--openrpc` file used by generation.
- TypeScript/Dart/Rust prototype sample drift: run `generate --target all` into a scratch directory and compare against generated package outputs; do not treat these samples as production SDK packaging evidence.
- Byte-level codec/signing drift: run `conformance-fixtures` and compare against `fixtures/conformance/transfer.json`.
- Whole-crate regression: run `cargo test -p graphene-spec-gen`.

Conformance fixture generation failures name the failed component as `operation_encoding`, `transaction_encoding`, `digest`, `key_signature`, or `json_envelope`.

## Validation and test coverage

The test suite validates the Swaplock and Acta fixtures plus broken variants for unknown API/type references, duplicate operation ids/names, duplicate codec fields, invalid chain ids, invalid callback references, empty transaction digest preimages, and classified shape evidence for approved raw fallbacks versus unsupported Graphene shapes. It also covers CLI behavior, lowering, OpenRPC enrichment, prototype emitter output, generated sample stability, and conformance fixture contents. The end-to-end CLI workflow intentionally validates and inspects both mandatory chain fixtures, then keeps prototype sample generation and conformance fixture emission in the same CI-safe loop without requiring live endpoints or language SDK packaging.

```sh
cargo test -p graphene-spec-gen
```

## Non-goals

OpenGraphene v0.1 deliberately does not yet define or ship:

- Full production TypeScript or Dart SDK runtimes.
- Generated transaction encoders, signers, WebSocket transports, or API clients.
- Full OpenRPC ingestion or reconciliation beyond the subset needed for current method metadata enrichment.
- Complete Graphene binary codec coverage beyond the current builtins, arrays, optionals, structs, operations, transactions, and reference conformance case.
- Runtime WebSocket dispatch, reconnect, unsubscribe, or callback lifecycle policy.
- Auth flows, wallet/key management UX, live broadcast side effects, or chain-specific secret handling.
- Automatic discovery of chain APIs from live nodes.
- Cross-chain compatibility guarantees beyond the validated contract document shape.

## Next milestone candidates

Useful follow-up slices include:

- Expand emitter outputs from metadata samples into runtime-safe TypeScript and Dart SDK modules.
- Generate transaction codec and signing helpers that are proven by `fixtures/conformance/transfer.json`.
- Add more conformance cases for additional operation variants, optional fields, arrays, extensions, and failure cases.
- Broaden OpenRPC reconciliation and report mismatches between OpenRPC methods and OpenGraphene method bindings.
- Add a versioned IR compatibility policy before external emitters depend on the JSON shape.
- Integrate generated callback metadata with the synchronous WebSocket dispatcher and handwritten callback helpers.
