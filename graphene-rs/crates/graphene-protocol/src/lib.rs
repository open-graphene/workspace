//! Core protocol primitives shared by Graphene-family chain crates.
//!
//! This crate owns runtime types and macros that generated chain crates use directly. Code that
//! reads C++ sources and emits Rust files belongs in `graphene-codegen`; the generated Rust should
//! depend on this crate for shared protocol behavior.

/// Defines a typed Graphene object-id wrapper for the current module.
///
/// This is intentionally only a compileable API skeleton for now. The generated modules can be
/// indexed by rust-analyzer and carry source-backed metadata, while the full `Id` wrapper behavior
/// is implemented in a later step.
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
#[macro_export]
macro_rules! define_object_id_type {
    (
        name: $name:literal,
        object_space: $object_space:literal,
        type_id: $type_id:literal $(,)?
    ) => {
        /// Source Graphene object-family name.
        pub const NAME: &str = $name;
        /// Graphene object space for this object-family.
        pub const OBJECT_SPACE: u8 = $object_space;
        /// Graphene type id inside [`OBJECT_SPACE`].
        pub const TYPE_ID: u8 = $type_id;
    };
}
