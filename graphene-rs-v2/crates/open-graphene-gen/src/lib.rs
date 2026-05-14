//! OpenGraphene contract model and validation.
//!
//! This crate is the seed for a language-neutral Graphene SDK contract. The
//! initial scope is intentionally small: deserialize an OpenGraphene document,
//! generate a JSON Schema for editor/CI use, and run domain validation before
//! later generator work consumes the document.

pub mod model;
pub mod validation;
