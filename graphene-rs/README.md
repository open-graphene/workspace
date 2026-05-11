# graphene-rs developer workflow

This workspace README is for contributors changing the generated BitShares model layer. After reading it, you should be able to regenerate BitShares generated models, verify that checked-in generated files are current, and decide whether raw or unsupported fallback rows are acceptable.

## Prerequisites and working directory

Run commands from the repository root unless a command says otherwise. The BitShares generators use only checked-in inputs: BitShares C++ source, codegen configuration, and tracked JSON fixtures. They do not require secrets or a live RPC node.

You need a working Rust toolchain with `cargo` and `rustfmt`, plus `git` so the freshness check can report generated-file drift.

## Regenerate BitShares models

Use the hardened one-command workflow when C++ protocol definitions, generator mappings, or generated BitShares files change:

```sh
bash graphene-v2/graphene-rs/crates/graphene-chain-bitshares/bin/gen.sh --write
```

That command runs the BitShares object ID, operation tag, and operation struct generators with the crate's checked-in codegen configuration. Generated BitShares operation structs and enums stay in `graphene-chain-bitshares`; shared `graphene-protocol` raw wrappers remain fallback primitives, not a home for chain-specific BitShares operation types.

The generated artifact set includes operation variants, typed operations, operation coverage reporting, and generated object ID modules under the BitShares chain crate. Treat these files as owned by the generator: change generator inputs or mappings, then regenerate, rather than hand-editing generated Rust or reports.

## Verify generated-file freshness

After regeneration, run the freshness check:

```sh
bash graphene-v2/graphene-rs/crates/graphene-chain-bitshares/bin/check-generated.sh
```

The script regenerates with `--write` and then runs a path-scoped `git diff --exit-code` over the generated BitShares outputs. A failure means the working tree contains stale generated files; the diff paths show which generated artifacts need to be committed or which generator change caused drift.

If you need to inspect manually, use `git diff -- graphene-v2/graphene-rs/crates/graphene-chain-bitshares/src/operation_variants.rs graphene-v2/graphene-rs/crates/graphene-chain-bitshares/src/operations.rs graphene-v2/graphene-rs/crates/graphene-chain-bitshares/src/operation_model_skips.md graphene-v2/graphene-rs/crates/graphene-chain-bitshares/src/types`.

## Proof commands

Run these checks before claiming a generator or generated-model change is complete:

```sh
cargo test --manifest-path graphene-v2/graphene-rs/Cargo.toml -p graphene-chain-bitshares --test operations
cargo test --manifest-path graphene-v2/graphene-rs/Cargo.toml -p graphene-chain-bitshares --test transaction
cargo test --manifest-path graphene-v2/graphene-rs/Cargo.toml -p graphene-chain-bitshares --test fixtures deserializes_all_captured_live_rpc_fixtures
cargo fmt --manifest-path graphene-v2/graphene-rs/Cargo.toml -p graphene-codegen -p graphene-chain-bitshares -- --check
```

Use generator stdout/stderr, cargo diagnostics, and `git diff --exit-code` output as the primary local failure signals. The fixture tests prove serde compatibility against tracked samples without contacting a live chain.

## Fallback policy

Typed BitShares operations should be generated whenever the BitShares C++ source can be mapped to protocol-safe Rust types. If a static-variant operation tag has a generated model, deserialization should produce the typed `Operation` variant and `Operation::is_typed()` should be true.

Raw field fallback is acceptable only for approved extension or static-variant payload shapes where preserving the Graphene JSON value is intentional. These rows are reported as `approved_raw_fallback` in `operation_model_skips.md` and usually preserve extension payloads through shared raw wrapper primitives.

`Operation::Unsupported` is the explicit fallback for skipped or unknown operation tags. It preserves the numeric tag and original payload so callers do not lose data, but it is not a substitute for generator work when the C++ type can be mapped safely. If a known operation row is classified as `unsupported`, add or fix the generator mapping and regenerate instead of manually editing the report.

## Operation coverage report

`graphene-v2/graphene-rs/crates/graphene-chain-bitshares/src/operation_model_skips.md` is generated and must not be hand-edited. It summarizes total operations, generated structs, skipped or unsupported rows, and approved raw fallback rows.

Read the classification column first:

- `approved_raw_fallback` means the generator intentionally preserved a raw extension/static-variant payload.
- `unsupported` means at least one field has no protocol-safe mapping yet, so the containing operation struct is skipped and that operation tag falls back to `Operation::Unsupported`.

To change the report, change the C++ source inputs or generator mappings, run the regeneration command, then run the freshness and proof commands.

## Fixture and null-skip contract

Tracked JSON fixtures are part of the source of truth for serde behavior. The fixture regression test deserializes every present fixture object through the generated BitShares models and asserts that operation-bearing fixtures use typed operations where coverage exists.

A fixture with `object: null` is allowed only when it also records a non-empty `skip_reason` or RPC error metadata. This keeps intentional fixture gaps visible and prevents silent loss of coverage. If the fixture test fails, fix the fixture metadata or the generated model contract; do not weaken the test to hide drift.
