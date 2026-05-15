# graphene-gen-spec

`graphene-gen-spec` generates an **open-graphene** chain bundle from a Graphene/FC C++ core checkout.

## CLI

```sh
cargo run -p graphene-gen-spec -- generate graphene/chains/swaplock.toml
cargo run -p graphene-gen-spec -- audit graphene/chains/swaplock.toml
```

The default config is `graphene/chains/swaplock.toml`.

## Bundle layout

Generated specs live under `graphene/specs/<chain>/`:

```text
graphene/specs/swaplock/
  open-graphene.json              # chain bundle manifest
  api.database.json               # graphene::app::database_api
  api.network-broadcast.json      # graphene::app::network_broadcast_api
```

The manifest is the entrypoint for future SDK generators. Surface specs remain separate because Graphene exposes multiple API namespaces over one websocket connection.

## Pipeline

```text
Graphene C++ core + chain config
  -> Doxygen XML for each selected FC_API class
  -> OpenRPC methods from FC_API + Doxygen
  -> OpenRPC components.schemas from FC_REFLECT/static_variant metadata
  -> api.<surface>.json OpenRPC surface specs
  -> open-graphene.json chain bundle manifest
```

The importer backend is vendored from the neutral part of the previous `graphene-codegen` crate:

- `openrpc/gen_openrpc.sh`
- `openrpc/gen_api_spec.py`
- `openrpc/gen_types_spec.py`

Rust binding generation is deliberately excluded from this crate.

## Manifest shape

```json
{
  "x-open-graphene": {
    "version": "0.1.0",
    "kind": "chain-bundle",
    "chain": "swaplock",
    "surfaces": [
      {
        "name": "swaplock-database",
        "kind": "database",
        "apiQualifiedName": "graphene::app::database_api",
        "spec": "api.database.json"
      },
      {
        "name": "swaplock-broadcast",
        "kind": "broadcast",
        "apiQualifiedName": "graphene::app::network_broadcast_api",
        "spec": "api.network-broadcast.json"
      }
    ]
  }
}
```

## Audit

`graphene-gen-spec audit` checks the manifest and every referenced surface spec:

- manifest `kind`, `version`, `chain`, and surface entries
- relative `spec` paths that cannot escape the bundle directory
- OpenRPC `1.3.2` surface specs
- `methods` and `components.schemas`
- local `#/components/schemas/*` references
- absence of `TODO` placeholder descriptions
- Graphene static variants encoded as `[index, payload]` with matching `x-graphene-static-variant` metadata
- Graphene containers encoded with neutral `x-graphene-container` metadata
- neutral `x-graphene-scalar` metadata on `GrapheneTimePointSec`, `GrapheneInt64`, and `GrapheneUInt64`

Legacy `x-rust-type` and `x-fc-container` metadata is rejected; language-specific bindings must map neutral Graphene metadata in their own backend.
