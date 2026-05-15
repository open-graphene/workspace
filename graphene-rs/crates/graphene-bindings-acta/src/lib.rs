//! Generated Rust data models for Acta.
//!
//! This crate contains schema structs and Graphene static_variant enums generated
//! from open-graphene specs. Transport and runtime SDK layers are intentionally
//! not generated here.

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
    include!("generated/metadata.rs");
}

pub use generated::*;
