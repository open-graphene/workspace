//! Rust binding generation for OpenGraphene specifications.
//!
//! This crate consumes the language-neutral model and IR from `graphene-spec-gen`, then emits
//! Rust-specific binding artifacts. Keep specification parsing, validation, and IR construction in
//! `graphene-spec-gen`; keep Rust code-generation details here.

pub mod spec_metadata;
