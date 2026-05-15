# graphene-gen-dart

`graphene-gen-dart` owns Dart-specific metadata generation for OpenGraphene contracts.

It consumes the language-neutral model and IR from `graphene-spec-gen`; it does not parse, validate, or publish OpenGraphene specs itself.

## Command

Run from the `graphene/` workspace root:

```sh
cargo run -p graphene-gen-dart -- generate \
  specs/swaplock.opengraphene.json \
  --openrpc crates/graphene-spec-gen/fixtures/swaplock.openrpc.json \
  --out /tmp/graphene-dart
```

The repository-level regeneration script delegates to this crate:

```sh
bin/gen_dart.sh
```

## Layout

```text
crates/graphene-gen-dart/
  src/lib.rs                         # Dart emitter from OpenGraphene IR
  src/bin/graphene-gen-dart.rs       # CLI entrypoint
  tests/snapshots/dart/              # emitter snapshots without OpenRPC enrichment
  fixtures/generated-samples/        # OpenRPC-enriched sample output
```
