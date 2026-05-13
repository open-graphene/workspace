//! Chain-local Rust types for RSquared.
//!
//! This crate is generated from the RSquared OpenRPC specification. Schema
//! structs come from `typify`, while Graphene static variants use custom serde
//! to preserve the `[index, payload]` wire shape.

#![allow(clippy::all)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::enum_variant_names)]

pub mod generated {
    include!("generated/types.rs");
    include!("generated/variants.rs");
    include!("generated/rpc.rs");
}

pub use generated::*;
