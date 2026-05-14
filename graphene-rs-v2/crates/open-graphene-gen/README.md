# open-graphene-gen

`open-graphene-gen` owns the OpenGraphene contract surface used by generator work. At this stage it validates contract documents and emits a JSON Schema for the contract model; code generation is intentionally not implemented yet.

## CLI

Run from the repository root workspace:

```sh
cargo run -p open-graphene-gen -- validate crates/open-graphene-gen/fixtures/swaplock.opengraphene.json
cargo run -p open-graphene-gen -- schema
cargo run -p open-graphene-gen -- conformance-fixtures --out crates/open-graphene-gen/fixtures/conformance
```

The installed/debug binary has the same subcommands:

```sh
open-graphene-gen validate <path>
open-graphene-gen schema
open-graphene-gen conformance-fixtures --out <dir>
```

`validate` prints `<path>: valid OpenGraphene contract` for valid input. Invalid documents are rejected with path-specific diagnostics such as `methodBindings.broadcast_transaction.api: references unknown API ...`.

`conformance-fixtures` writes byte-level Rust reference fixtures such as `transfer.json` into the output directory. Generation failures identify the failed component as `operation_encoding`, `transaction_encoding`, `digest`, `key_signature`, or `json_envelope`, which lets downstream agents diagnose whether drift came from operation encoding, transaction encoding, digest construction, key/signature handling, or JSON broadcast envelope assembly.

## OpenGraphene v0.1

OpenGraphene v0.1 is a chain contract document that describes enough Graphene RPC, codec, transaction, operation, and callback semantics for downstream SDK/code generator work without forcing contributors to read Rust structs.

The current representative fixture is:

```text
crates/open-graphene-gen/fixtures/swaplock.opengraphene.json
```

It is a Swaplock-style contract used by validation tests and should be treated as the canonical compact example for the current schema.

### Root fields

A v0.1 document contains these top-level sections:

- `openGraphene`: contract version string. Current fixtures use `"0.1"`.
- `openrpc`: optional path to the matching OpenRPC description.
- `chain`: chain metadata such as name, chain id, and endpoints.
- `apis`: named Graphene API surfaces.
- `methodBindings`: mapping from OpenRPC method names to Graphene API surfaces.
- `codec`: named binary codec types needed by methods, operations, transactions, and callback payloads.
- `operations`: operation variant sets with numeric ids and payload types.
- `transaction`: signed transaction semantics.
- `callbacks`: callback-taking RPC contracts that need generated or handwritten client handling.

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

`methodBindings` assigns each OpenRPC method to one declared API. This keeps the OpenRPC method shape separate from Graphene's API routing.

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

Validation rejects bindings that reference an unknown `apis` key and reports the failing path.

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

Builtins currently include `bool`, `bytes`, integer widths used by Graphene (`uint8`, `uint16`, `uint32`, `uint64`, `int64`), `object_id`, `public_key`, `signature`, `string`, `time_point_sec`, and `void`. Validation rejects duplicate fields and unknown type references.

### `operations`

`operations` declares variant sets. Each variant has a Graphene operation id, a stable operation name, and the codec payload type.

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

### Conformance fixtures

Conformance fixtures are stable JSON files generated from the Rust reference codec and signer. They are intended for non-Rust native implementations, currently TypeScript and Dart, to prove byte-for-byte compatibility before those SDKs own their own transaction codec and signing code.

Generate the checked-in reference fixtures with:

```sh
cargo run -p open-graphene-gen -- conformance-fixtures --out crates/open-graphene-gen/fixtures/conformance
```

The current canonical fixture is:

```text
crates/open-graphene-gen/fixtures/conformance/transfer.json
```

Each fixture has this contract:

- `schemaVersion`: fixture schema version. Consumers should reject unknown major versions instead of silently accepting changed semantics.
- `name`: stable test case name, for example `swaplock-transfer`.
- `chainId`: chain id used in the signing digest preimage.
- `input.operationName`: operation variant under test.
- `input.operationJson`: JSON-ish operation payload that a generated SDK should accept before binary encoding.
- `input.transactionJson`: unsigned transaction JSON that wraps the operation and supplies header fields such as ref block and expiration.
- `input.privateKeyWif`: deterministic fixture key used only for conformance tests.
- `expected.operationHex`: expected Graphene operation bytes, including the static-variant operation id and encoded payload. A mismatch here isolates drift to operation variant or field encoding.
- `expected.transactionHex`: expected unsigned transaction hex. This proves transaction header, operation vector, extensions, and nested operation bytes are encoded in the Rust reference order.
- `expected.digestHex`: expected signing digest over `chainId || transaction bytes`. This distinguishes serialization drift from digest-preimage drift.
- `expected.publicKey`: public key derived from `input.privateKeyWif`, proving the consumer is interpreting fixture key material consistently.
- `expected.signatureHex`: expected compact recoverable Graphene signature bytes for the digest.
- `expected.signatureMetadata`: digest algorithm, curve, signature format, and canonical-signature requirement that TypeScript and Dart signers must enforce.
- `expected.broadcastPayload`: JSON-RPC broadcast method, API name, and signed transaction JSON envelope expected after the signature is attached.

TypeScript and Dart conformance tests should load the fixture, build the operation and unsigned transaction from `input`, and compare each output field independently in the same order: operation hex, transaction hex, digest hex, public key, signature hex, then broadcast payload. Keeping these assertions separate makes byte-level drift actionable: an operation mismatch points at the operation codec, a transaction mismatch points at transaction framing, a digest mismatch points at preimage construction, a signature mismatch points at key/signing behavior, and a broadcast-payload mismatch points at JSON envelope assembly.

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

## Current non-goals

OpenGraphene v0.1 deliberately does not yet define:

- Rust/TypeScript code generation output.
- Full OpenRPC ingestion or reconciliation with the `openrpc` path.
- Complete Graphene binary codec coverage beyond the current builtins and `struct` shape.
- Runtime WebSocket dispatch, reconnect, unsubscribe, or callback lifecycle policy.
- Auth, key management, signing implementation, or broadcast side effects.
- Cross-chain schema compatibility guarantees beyond the validated contract document shape.

## Validation coverage

The test suite validates the Swaplock fixture and broken variants for unknown API/type references, duplicate operation ids/names, duplicate codec fields, invalid chain ids, invalid callback references, and empty transaction digest preimages.

```sh
cargo test -p open-graphene-gen
```
