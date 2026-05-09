/// Defines a chain-specific typed object id wrapper around [`ObjectId`].
///
/// The macro intentionally requires each chain crate to provide the object type number and
/// chain-specific wording. That keeps the numeric object map owned by the chain crate while sharing
/// the repetitive wrapper implementation.
#[macro_export]
macro_rules! define_object_id_type {
    (
        chain: $chain_name:literal,
        chain_article: $chain_article:literal,
        $(#[$id_meta:meta])*
        id: $type_name:literal,
        type_id: $type_id:literal,
        instance_doc: $instance_doc:literal,
        wrong_type_doc: $wrong_type_doc:literal,
        wrong_type_message: $wrong_type_message:literal $(,)?
    ) => {
        use core::fmt;
        use core::str::FromStr;

        use $crate::{ObjectId, ObjectIdParseError};

        const SPACE: u8 = 1;
        const TYPE_ID: u8 = $type_id;

        $(#[$id_meta])*
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct Id(ObjectId);

        impl Id {
            #[doc = concat!("Creates ", $chain_article, " ", $chain_name, " ", $type_name, " id from its object instance component.")]
            pub const fn new(instance: u64) -> Self {
                Self(ObjectId::new(SPACE, TYPE_ID, instance))
            }

            /// Returns the underlying untyped Graphene object id.
            pub const fn object_id(self) -> ObjectId {
                self.0
            }

            #[doc = $instance_doc]
            pub const fn instance(self) -> u64 {
                self.0.instance()
            }
        }

        impl fmt::Display for Id {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<Id> for ObjectId {
            fn from(value: Id) -> Self {
                value.object_id()
            }
        }

        impl TryFrom<ObjectId> for Id {
            type Error = Error;

            fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
                if value.space() == SPACE && value.type_id() == TYPE_ID {
                    return Ok(Self(value));
                }

                Err(Error::WrongType { actual: value })
            }
        }

        impl FromStr for Id {
            type Err = Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::try_from(value.parse::<ObjectId>()?)
            }
        }

        #[doc = concat!("Error returned when parsing or converting ", $chain_article, " ", $chain_name, " ", $type_name, " id fails.")]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum Error {
            /// The underlying object id string could not be parsed.
            Parse(ObjectIdParseError),
            #[doc = $wrong_type_doc]
            WrongType {
                /// The parsed object id with the unexpected type.
                actual: ObjectId,
            },
        }

        impl fmt::Display for Error {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Self::Parse(error) => error.fmt(f),
                    Self::WrongType { actual } => write!(f, $wrong_type_message, actual = actual),
                }
            }
        }

        impl From<ObjectIdParseError> for Error {
            fn from(error: ObjectIdParseError) -> Self {
                Self::Parse(error)
            }
        }
    };
}
