//! Core protocol primitives shared by Graphene-family chain crates.
//!
//! This crate owns runtime types and macros that generated chain crates use directly. Code that
//! reads C++ sources and emits Rust files belongs in `graphene-codegen`; the generated Rust should
//! depend on this crate for shared protocol behavior.

pub mod asset;
pub mod authority;
pub mod chain_parameters;
pub mod object_id;

pub use asset::Asset;
pub use authority::Authority;
pub use chain_parameters::ImmutableChainParameters;
pub use object_id::{ObjectId, ObjectIdParseError};
