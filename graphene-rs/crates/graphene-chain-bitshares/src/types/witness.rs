use core::fmt;
use core::str::FromStr;

use graphene_protocol::{ObjectId, ObjectIdParseError};

const SPACE: u8 = 1;
const TYPE_ID: u8 = 6;

/// A BitShares witness object identifier (`1.6.x`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(ObjectId);

impl Id {
    /// Creates a BitShares witness id from its object instance component.
    pub const fn new(instance: u64) -> Self {
        Self(ObjectId::new(SPACE, TYPE_ID, instance))
    }

    /// Returns the underlying untyped Graphene object id.
    pub const fn object_id(self) -> ObjectId {
        self.0
    }

    /// Returns the witness object instance component.
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

/// Error returned when parsing or converting a BitShares witness id fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The underlying object id string could not be parsed.
    Parse(ObjectIdParseError),
    /// The object id is not in the BitShares witness object range (`1.6.x`).
    WrongType {
        /// The parsed object id with the unexpected type.
        actual: ObjectId,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => error.fmt(f),
            Self::WrongType { actual } => {
                write!(f, "object id {actual} is not a BitShares witness id")
            }
        }
    }
}

impl From<ObjectIdParseError> for Error {
    fn from(error: ObjectIdParseError) -> Self {
        Self::Parse(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, Id};
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_witness_ids_from_instances() {
        let witness = Id::new(11);

        assert_eq!(witness.object_id(), ObjectId::new(1, 6, 11));
        assert_eq!(ObjectId::from(witness), ObjectId::new(1, 6, 11));
        assert_eq!(Id::try_from(ObjectId::new(1, 6, 11)), Ok(witness));
    }

    #[test]
    fn rejects_non_witness_object_ids() {
        assert_eq!(
            "1.5.11".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(1, 5, 11),
            })
        );
    }

    #[test]
    fn propagates_object_id_parse_errors() {
        assert_eq!(
            "not-an-id".parse::<Id>(),
            Err(Error::Parse(ObjectIdParseError::WrongPartCount))
        );
    }
}
