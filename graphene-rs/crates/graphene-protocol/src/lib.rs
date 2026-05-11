//! Core protocol primitives shared by Graphene-family chain crates.
//!
//! This crate owns runtime types and macros that generated chain crates use directly. Code that
//! reads C++ sources and emits Rust files belongs in `graphene-codegen`; the generated Rust should
//! depend on this crate for shared protocol behavior.

/// Defines a typed Graphene object-id wrapper for the current module.
///
/// This is intentionally only the API skeleton for now. The generator can target this call shape
/// before the wrapper implementation is filled in.
///
/// Object family name, object space, and type id must be explicit. Generated code should preserve
/// the exact facts read from Graphene `GRAPHENE_DEFINE_IDS(...)` declarations instead of relying on
/// defaults or module-name inference.
///
/// ```ignore
/// graphene_protocol::define_object_id_type! {
///     name: "account",
///     object_space: 1,
///     type_id: 2,
/// }
/// ```
///
/// ```ignore
/// graphene_protocol::define_object_id_type! {
///     name: "account_balance",
///     object_space: 2,
///     type_id: 5,
/// }
/// ```
#[macro_export]
macro_rules! define_object_id_type {
    (
        name: $name:literal,
        object_space: $object_space:literal,
        type_id: $type_id:literal $(,)?
    ) => {
        compile_error!(
            "graphene_protocol::define_object_id_type! is an API skeleton; wrapper emission is not implemented yet"
        );
    };
}
