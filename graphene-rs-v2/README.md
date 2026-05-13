# graphene-rs-v2

Clean-room workspace for the next Graphene Rust SDK/codegen architecture.

The current goal is to make the Swaplock OpenRPC/Rust-types POC repeatable for
any Graphene-like blockchain before building higher-level SDK ergonomics.

## Current contents

```text
crates/graphene-codegen/
  src/bin/openrpc-typify.rs   # explicit typify runner: schema.json -> types.rs
  openrpc/                    # explicit OpenRPC generation pipeline scripts
    generate_from_config.py   # TOML-driven runner for the pipeline below
    gen_wallet_openrpc.sh     # wallet.hpp + C++ headers -> OpenRPC spec pipeline
    gen_wallet_spec.py        # Doxygen wallet_api XML -> OpenRPC methods
    gen_types_spec.py         # FC_REFLECT/static_variant scan -> OpenRPC schemas
    extract_typify_schema.py  # OpenRPC schemas -> JSON Schema $defs for typify
    gen_rust_variants.py      # static_variant schemas -> Rust [tag, payload] enums
    gen_rust_rpc.py           # OpenRPC methods -> Rust params/response bindings
crates/graphene-chain-swaplock/
  src/generated/              # checked generated Swaplock chain-local types
```

`graphene-codegen/openrpc` is intentionally still script-based. The scripts are
hidden under the codegen crate instead of root-level `tools/` so generation
policy has a clear owner while we continue validating behavior. The typify runner
has been folded into the same crate as the `openrpc-typify` binary.

## Generic Swaplock example

Run the configured pipeline:

```sh
./crates/graphene-codegen/openrpc/generate_from_config.py \
  --config examples/swaplock-openrpc.toml
```

For faster iteration when `spec.json` already exists, skip the Doxygen/C++ pass:

```sh
./crates/graphene-codegen/openrpc/generate_from_config.py \
  --config examples/swaplock-openrpc.toml \
  --skip-openrpc
```

Generate/fill an OpenRPC spec from a chain core directly:

```sh
./crates/graphene-codegen/openrpc/gen_wallet_openrpc.sh \
  --chain-name swaplock \
  --core-root /Users/lunacrafts/workspaces/mirrorboards/boards/boards-swaplock/swaplock-core \
  --output /tmp/swaplock-spec.json
```

Extract a typify-compatible JSON Schema document from an existing OpenRPC spec:

```sh
./crates/graphene-codegen/openrpc/extract_typify_schema.py \
  --spec /Users/lunacrafts/workspaces/mirrorboards/boards/boards-swaplock/swaplock-core/bin/dist/spec.json \
  --output /tmp/swaplock-schema.json \
  --title swaplock_types_root
```

Generate Rust static-variant enums from OpenRPC schemas:

```sh
./crates/graphene-codegen/openrpc/gen_rust_variants.py \
  --spec /Users/lunacrafts/workspaces/mirrorboards/boards/boards-swaplock/swaplock-core/bin/dist/spec.json \
  --out-rs /tmp/swaplock-variants.rs \
  --out-names /tmp/swaplock-variants.names \
  --source-label bin/dist/spec.json
```

Run the typify backend directly:

```sh
cargo run -p graphene-codegen --bin openrpc-typify -- \
  --schema /tmp/swaplock-schema.json \
  --out /tmp/types.rs \
  --strip-names /tmp/variants.names
```

## Next steps

1. Add a repeatable generated-output audit command under `graphene-codegen`.
2. Port the Python Rust-facing generators into Rust modules incrementally.
3. Keep C++/Doxygen import logic isolated from SDK runtime crates.
4. Build the typed RPC SDK only after the codegen ownership boundary is stable.
