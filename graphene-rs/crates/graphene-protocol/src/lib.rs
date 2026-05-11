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
/// Protocol-space objects may omit `object_space`; they default to Graphene object space `1`.
///
/// ```ignore
/// graphene_protocol::define_object_id_type! {
///     type_id: 2,
/// }
/// ```
///
/// Implementation-space objects pass `object_space: 2` explicitly.
///
/// ```ignore
/// graphene_protocol::define_object_id_type! {
///     object_space: 2,
///     type_id: 5,
/// }
/// ```
///
/// The source object-family name should live in the module/file name generated around this macro,
/// not in the macro invocation itself. For example, `types/account_balance.rs` containing
/// `object_space: 2, type_id: 5` is enough information to define `account_balance::Id`.
#[macro_export]
macro_rules! define_object_id_type {
    (
        type_id: $type_id:literal $(,)?
    ) => {
        $crate::define_object_id_type! {
            object_space: 1,
            type_id: $type_id,
        }
    };

    (
        object_space: $object_space:literal,
        type_id: $type_id:literal $(,)?
    ) => {
        compile_error!(
            "graphene_protocol::define_object_id_type! is an API skeleton; wrapper emission is not implemented yet"
        );
    };
}
