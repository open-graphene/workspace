//! Core protocol primitives shared by Graphene-family chain crates.
//!
//! This crate owns runtime types and macros that generated chain crates use directly. Code that
//! reads C++ sources and emits Rust files belongs in `graphene-codegen`; the generated Rust should
//! depend on this crate for shared protocol behavior.

use core::fmt;
use core::str::FromStr;

/// Minimal placeholder for Graphene `time_point_sec` values.
pub type TimePointSec = String;

/// Untyped Graphene object id in `space.type.instance` form.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectId {
    space: u8,
    type_id: u8,
    instance: u64,
}

impl ObjectId {
    /// Creates an object id from its three wire components.
    pub const fn new(space: u8, type_id: u8, instance: u64) -> Self {
        Self {
            space,
            type_id,
            instance,
        }
    }

    /// Returns the object space component.
    pub const fn space(self) -> u8 {
        self.space
    }

    /// Returns the object type component.
    pub const fn type_id(self) -> u8 {
        self.type_id
    }

    /// Returns the object instance component.
    pub const fn instance(self) -> u64 {
        self.instance
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.space, self.type_id, self.instance)
    }
}

impl FromStr for ObjectId {
    type Err = ObjectIdParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split('.');
        let Some(space) = parts.next() else {
            return Err(ObjectIdParseError::WrongPartCount);
        };
        let Some(type_id) = parts.next() else {
            return Err(ObjectIdParseError::WrongPartCount);
        };
        let Some(instance) = parts.next() else {
            return Err(ObjectIdParseError::WrongPartCount);
        };
        if parts.next().is_some() {
            return Err(ObjectIdParseError::WrongPartCount);
        }

        Ok(Self {
            space: parse_u8_part(space)?,
            type_id: parse_u8_part(type_id)?,
            instance: instance
                .parse()
                .map_err(|_| ObjectIdParseError::InvalidInteger)?,
        })
    }
}

fn parse_u8_part(value: &str) -> Result<u8, ObjectIdParseError> {
    value
        .parse()
        .map_err(|_| ObjectIdParseError::InvalidInteger)
}

/// Error returned when parsing a Graphene object id fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectIdParseError {
    /// The id did not contain exactly three dot-separated parts.
    WrongPartCount,
    /// One id component was not a valid integer for its target range.
    InvalidInteger,
}

impl fmt::Display for ObjectIdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPartCount => write!(f, "object id must have space.type.instance parts"),
            Self::InvalidInteger => write!(f, "object id contains an invalid integer component"),
        }
    }
}

/// Defines a typed Graphene object-id wrapper for the current module.
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

        /// Typed object id for this Graphene object-family.
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct Id($crate::ObjectId);

        impl Id {
            /// Creates a typed id from the object instance component.
            pub const fn new(instance: u64) -> Self {
                Self($crate::ObjectId::new(OBJECT_SPACE, TYPE_ID, instance))
            }

            /// Returns the underlying untyped object id.
            pub const fn object_id(self) -> $crate::ObjectId {
                self.0
            }

            /// Returns the object instance component.
            pub const fn instance(self) -> u64 {
                self.0.instance()
            }
        }

        impl core::fmt::Display for Id {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<Id> for $crate::ObjectId {
            fn from(value: Id) -> Self {
                value.object_id()
            }
        }

        impl TryFrom<$crate::ObjectId> for Id {
            type Error = Error;

            fn try_from(value: $crate::ObjectId) -> Result<Self, Self::Error> {
                if value.space() == OBJECT_SPACE && value.type_id() == TYPE_ID {
                    return Ok(Self(value));
                }

                Err(Error::WrongType { actual: value })
            }
        }

        impl core::str::FromStr for Id {
            type Err = Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::try_from(value.parse::<$crate::ObjectId>()?)
            }
        }

        impl<'de> serde::Deserialize<'de> for Id {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error as _;

                let value = String::deserialize(deserializer)?;
                value.parse::<Self>().map_err(D::Error::custom)
            }
        }

        /// Error returned when parsing or converting this typed object id fails.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum Error {
            /// The underlying object id string could not be parsed.
            Parse($crate::ObjectIdParseError),
            /// The parsed object id belongs to a different object family.
            WrongType {
                /// The parsed object id with the unexpected object family.
                actual: $crate::ObjectId,
            },
        }

        impl core::fmt::Display for Error {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Self::Parse(error) => error.fmt(f),
                    Self::WrongType { actual } => write!(
                        f,
                        "object id {actual} is not a {name} id",
                        name = NAME.replace('_', " ")
                    ),
                }
            }
        }

        impl From<$crate::ObjectIdParseError> for Error {
            fn from(error: $crate::ObjectIdParseError) -> Self {
                Self::Parse(error)
            }
        }
    };
}
