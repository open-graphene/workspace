//! Code generation workspace for Graphene chain crates.
//!
//! The active OpenRPC pipeline still lives in `openrpc/` as explicit scripts so
//! it can be validated before being ported into Rust modules. Keeping those
//! scripts under this crate makes `graphene-codegen` the owner of generation
//! policy without exposing a root-level `tools/` workspace API.
