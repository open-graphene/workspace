# graphene-gen-rust

`graphene-gen-rust` owns Rust-specific binding generation for OpenGraphene contracts.

It deliberately depends on `graphene-spec-gen` for the language-neutral contract model, validation, OpenRPC enrichment, and lowered IR. Keep specification generation and IR construction in `graphene-spec-gen`; keep Rust code emission, full OpenRPC Rust helper scripts, and Rust binding fixtures here.

## Responsibilities

- Generate Rust spec metadata from an OpenGraphene contract plus compact OpenRPC metadata.
- Run the `typify` OpenRPC schema conversion used by Rust binding generation.
- Store full OpenRPC fixtures used only by Rust binding generation.
- Store Python helper scripts that emit Rust RPC and operation-variant binding modules from full OpenRPC descriptions.

## Commands

Run from the `graphene/` workspace root.

```sh
cargo run -p graphene-gen-rust --bin graphene-gen-rust-spec-metadata -- \
  specs/swaplock.opengraphene.json \
  --openrpc crates/graphene-spec-gen/fixtures/swaplock.openrpc.json \
  --out /tmp/swaplock-spec_metadata.rs

cargo run -p graphene-gen-rust --bin openrpc-typify -- \
  --schema /tmp/database.schema.json \
  --out /tmp/types.rs \
  --strip-names /tmp/variants.names
```

The repository-level Rust binding regeneration script delegates to this crate:

```sh
bin/gen_rs.sh
```

## Layout

```text
crates/graphene-gen-rust/
  fixtures/openrpc-full/      # full OpenRPC inputs for Rust binding generation
  openrpc/                    # Rust binding helper scripts for full OpenRPC specs
  src/spec_metadata.rs        # Rust metadata emitter from OpenGraphene IR
  src/bin/openrpc-typify.rs   # typify wrapper for generated Rust OpenRPC types
  src/bin/spec-metadata.rs    # CLI for Rust spec_metadata.rs output
```

## Non-goals

- Parsing or validating OpenGraphene contracts directly; use `graphene-spec-gen`.
- Owning TypeScript or Dart emission.
- Replacing the language-neutral IR with Rust-specific metadata.
