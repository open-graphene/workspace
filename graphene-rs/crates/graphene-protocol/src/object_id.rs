use core::fmt;
use core::str::FromStr;

/// A Graphene object identifier in `space.type.instance` form, for example `1.2.345`.
///
/// This type deliberately does not assign semantic meaning to the `space.type` pair. Supported
/// chains can map object ids to chain-specific typed ids in their own crates.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectId {
    space: u8,
    type_id: u8,
    instance: u64,
}

impl ObjectId {
    /// Creates an object identifier from its numeric components.
    pub const fn new(space: u8, type_id: u8, instance: u64) -> Self {
        Self {
            space,
            type_id,
            instance,
        }
    }

    /// Returns the Graphene object space component.
    pub const fn space(self) -> u8 {
        self.space
    }

    /// Returns the object type component within the object space.
    pub const fn type_id(self) -> u8 {
        self.type_id
    }

    /// Returns the object instance component.
    pub const fn instance(self) -> u64 {
        self.instance
    }
}

/// Error returned when parsing a Graphene object identifier fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectIdParseError {
    /// The input did not contain exactly three dot-separated parts.
    WrongPartCount,
    /// One of the three dot-separated parts was empty.
    EmptyPart,
    /// One of the parts was not a base-10 unsigned integer.
    InvalidNumber,
    /// The `space` component did not fit in `u8`.
    SpaceOutOfRange,
    /// The `type` component did not fit in `u8`.
    TypeOutOfRange,
    /// The `instance` component did not fit in `u64`.
    InstanceOutOfRange,
}

impl fmt::Display for ObjectIdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPartCount => {
                f.write_str("object id must have exactly three dot-separated parts")
            }
            Self::EmptyPart => f.write_str("object id parts must not be empty"),
            Self::InvalidNumber => f.write_str("object id parts must be unsigned decimal integers"),
            Self::SpaceOutOfRange => f.write_str("object id space component must fit in u8"),
            Self::TypeOutOfRange => f.write_str("object id type component must fit in u8"),
            Self::InstanceOutOfRange => f.write_str("object id instance component must fit in u64"),
        }
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
        let space = parts.next().ok_or(ObjectIdParseError::WrongPartCount)?;
        let type_id = parts.next().ok_or(ObjectIdParseError::WrongPartCount)?;
        let instance = parts.next().ok_or(ObjectIdParseError::WrongPartCount)?;

        if parts.next().is_some() {
            return Err(ObjectIdParseError::WrongPartCount);
        }

        if space.is_empty() || type_id.is_empty() || instance.is_empty() {
            return Err(ObjectIdParseError::EmptyPart);
        }

        let space = parse_u8_part(space, ObjectIdParseError::SpaceOutOfRange)?;
        let type_id = parse_u8_part(type_id, ObjectIdParseError::TypeOutOfRange)?;
        let instance = parse_u64_part(instance).map_err(|error| match error {
            NumberPartError::InvalidNumber => ObjectIdParseError::InvalidNumber,
            NumberPartError::OutOfRange => ObjectIdParseError::InstanceOutOfRange,
        })?;

        Ok(Self::new(space, type_id, instance))
    }
}

fn parse_u8_part(value: &str, range_error: ObjectIdParseError) -> Result<u8, ObjectIdParseError> {
    let parsed = parse_u64_part(value).map_err(|error| match error {
        NumberPartError::InvalidNumber => ObjectIdParseError::InvalidNumber,
        NumberPartError::OutOfRange => range_error,
    })?;

    u8::try_from(parsed).map_err(|_| range_error)
}

fn parse_u64_part(value: &str) -> Result<u64, NumberPartError> {
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(NumberPartError::InvalidNumber);
    }

    value
        .parse::<u64>()
        .map_err(|_| NumberPartError::OutOfRange)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NumberPartError {
    InvalidNumber,
    OutOfRange,
}

#[cfg(test)]
mod tests {
    use super::{ObjectId, ObjectIdParseError};

    #[test]
    fn parses_object_id_parts() {
        let id: ObjectId = "1.2.345".parse().expect("valid object id should parse");

        assert_eq!(id.space(), 1);
        assert_eq!(id.type_id(), 2);
        assert_eq!(id.instance(), 345);
    }

    #[test]
    fn formats_object_id_parts() {
        let id = ObjectId::new(1, 2, 345);

        assert_eq!(id.to_string(), "1.2.345");
    }

    #[test]
    fn rejects_inputs_with_wrong_part_count() {
        assert_eq!(
            "1.2".parse::<ObjectId>(),
            Err(ObjectIdParseError::WrongPartCount)
        );
        assert_eq!(
            "1.2.3.4".parse::<ObjectId>(),
            Err(ObjectIdParseError::WrongPartCount)
        );
    }

    #[test]
    fn rejects_inputs_with_empty_parts() {
        assert_eq!(
            "1..3".parse::<ObjectId>(),
            Err(ObjectIdParseError::EmptyPart)
        );
        assert_eq!(
            ".2.3".parse::<ObjectId>(),
            Err(ObjectIdParseError::EmptyPart)
        );
        assert_eq!(
            "1.2.".parse::<ObjectId>(),
            Err(ObjectIdParseError::EmptyPart)
        );
    }

    #[test]
    fn rejects_non_numeric_parts() {
        assert_eq!(
            "x.2.3".parse::<ObjectId>(),
            Err(ObjectIdParseError::InvalidNumber)
        );
        assert_eq!(
            "1.x.3".parse::<ObjectId>(),
            Err(ObjectIdParseError::InvalidNumber)
        );
        assert_eq!(
            "1.2.x".parse::<ObjectId>(),
            Err(ObjectIdParseError::InvalidNumber)
        );
        assert_eq!(
            "+1.2.3".parse::<ObjectId>(),
            Err(ObjectIdParseError::InvalidNumber)
        );
        assert_eq!(
            "1.-2.3".parse::<ObjectId>(),
            Err(ObjectIdParseError::InvalidNumber)
        );
    }

    #[test]
    fn rejects_out_of_range_parts() {
        assert_eq!(
            "256.2.3".parse::<ObjectId>(),
            Err(ObjectIdParseError::SpaceOutOfRange)
        );
        assert_eq!(
            "1.256.3".parse::<ObjectId>(),
            Err(ObjectIdParseError::TypeOutOfRange)
        );
        assert_eq!(
            "1.2.18446744073709551616".parse::<ObjectId>(),
            Err(ObjectIdParseError::InstanceOutOfRange)
        );
    }
}
