# graphene-gen-ts

`graphene-gen-ts` owns TypeScript-specific metadata generation for OpenGraphene contracts.

It consumes the language-neutral model and IR from `graphene-spec-gen`; it does not parse, validate, or publish OpenGraphene specs itself.

## Command

Run from the `graphene/` workspace root:

```sh
cargo run -p graphene-gen-ts -- generate \
  specs/swaplock.opengraphene.json \
  --openrpc crates/graphene-spec-gen/fixtures/swaplock.openrpc.json \
  --out /tmp/graphene-ts
```

The repository-level regeneration script delegates to this crate:

```sh
bin/gen_ts.sh
```

## Layout

```text
crates/graphene-gen-ts/
  src/lib.rs                         # TypeScript emitter from OpenGraphene IR
  src/bin/graphene-gen-ts.rs         # CLI entrypoint
  tests/snapshots/typescript/        # emitter snapshots without OpenRPC enrichment
  fixtures/generated-samples/        # OpenRPC-enriched sample output
```
