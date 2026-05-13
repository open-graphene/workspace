# graphene-rs-v2

Clean-room workspace for the next Graphene Rust SDK/codegen architecture.

This directory starts with the generic pieces from the Swaplock OpenRPC/Rust-types POC, moved into project-owned tooling. The current goal is not to generate a final SDK yet; it is to make the POC repeatable for any Graphene-like blockchain.

## Current contents

```text
tools/openrpc/
  generate_from_config.py    # TOML-driven runner for the pipeline below
  gen_wallet_openrpc.sh      # generic wallet.hpp + C++ headers -> OpenRPC spec pipeline
  gen_wallet_spec.py         # Doxygen wallet_api XML -> OpenRPC methods
  gen_types_spec.py          # FC_REFLECT/static_variant C++ scan -> OpenRPC components.schemas
  extract_typify_schema.py   # OpenRPC components.schemas -> JSON Schema $defs for typify
  gen_rust_variants.py       # OpenRPC static_variant schemas -> Rust enums with [tag, payload] serde
crates/openrpc-typify/       # explicit typify runner: schema.json -> types.rs
```

The Python files are still based on the working POC. They are intentionally kept as scripts for this first step so we can validate behavior before porting pieces into Rust. Typify invocation has already been moved out of chain-crate `build.rs` into the explicit `openrpc-typify` runner.

## Generic Swaplock example

Run the configured pipeline:

```sh
./tools/openrpc/generate_from_config.py --config examples/swaplock-openrpc.toml
```

For faster iteration when `spec.json` already exists, skip the Doxygen/C++ pass:

```sh
./tools/openrpc/generate_from_config.py \
  --config examples/swaplock-openrpc.toml \
  --skip-openrpc
```

Generate/fill an OpenRPC spec from a chain core directly:

```sh
./tools/openrpc/gen_wallet_openrpc.sh \
  --chain-name swaplock \
  --core-root /Users/lunacrafts/workspaces/mirrorboards/boards/boards-swaplock/swaplock-core \
  --output /tmp/swaplock-spec.json
```

Extract a typify-compatible JSON Schema document from an existing OpenRPC spec:

```sh
./tools/openrpc/extract_typify_schema.py \
  --spec /Users/lunacrafts/workspaces/mirrorboards/boards/boards-swaplock/swaplock-core/bin/dist/spec.json \
  --output /tmp/swaplock-schema.json \
  --title swaplock_types_root
```

Generate Rust static-variant enums from OpenRPC schemas:

```sh
./tools/openrpc/gen_rust_variants.py \
  --spec /Users/lunacrafts/workspaces/mirrorboards/boards/boards-swaplock/swaplock-core/bin/dist/spec.json \
  --out-rs /tmp/swaplock-variants.rs \
  --out-names /tmp/swaplock-variants.names \
  --source-label bin/dist/spec.json
```

## Next steps

1. Create a generated `graphene-chain-swaplock` crate inside this workspace.
2. Add a Rust `graphene-codegen` crate that invokes these stages without the Python orchestration layer.
3. Split or organize generated `types.rs` into reviewable chain-local modules.
