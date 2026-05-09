use core::fmt;
use core::str::FromStr;

const MAX_INSTANCE: u32 = 0x00ff_ffff;

/// BitShares vote target kind.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    /// Committee-member vote target (`0`).
    Committee,
    /// Witness vote target (`1`).
    Witness,
    /// Worker vote target (`2`).
    Worker,
}

impl Kind {
    /// Returns the numeric vote type used in JSON vote ids.
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Committee => 0,
            Self::Witness => 1,
            Self::Worker => 2,
        }
    }
}

impl TryFrom<u8> for Kind {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Committee),
            1 => Ok(Self::Witness),
            2 => Ok(Self::Worker),
            _ => Err(Error::UnknownKind(value)),
        }
    }
}

/// A BitShares vote identifier serialized in JSON as `type:instance`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id {
    kind: Kind,
    instance: u32,
}

impl Id {
    /// Creates a vote id for a known vote kind and 24-bit instance.
    ///
    /// Panics when `instance` exceeds the 24-bit Graphene vote-id instance range.
    pub const fn new(kind: Kind, instance: u32) -> Self {
        assert!(instance <= MAX_INSTANCE, "vote id instance exceeds 24 bits");
        Self { kind, instance }
    }

    /// Returns the vote target kind.
    pub const fn kind(self) -> Kind {
        self.kind
    }

    /// Returns the vote target instance.
    pub const fn instance(self) -> u32 {
        self.instance
    }

    /// Returns the packed Graphene wire representation.
    pub const fn content(self) -> u32 {
        (self.instance << 8) | self.kind.as_u8() as u32
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind.as_u8(), self.instance)
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        parse_vote_id(value)
    }
}

impl<'de> serde::Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;

        let value = <&str>::deserialize(deserializer)?;
        value.parse::<Self>().map_err(D::Error::custom)
    }
}

/// Error returned when parsing a BitShares vote id fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The vote id did not contain exactly one colon separator.
    WrongPartCount,
    /// The vote kind or instance part was empty.
    EmptyPart,
    /// The vote kind was not numeric.
    InvalidKind,
    /// The vote kind is outside the known BitShares range.
    UnknownKind(u8),
    /// The vote instance was not numeric.
    InvalidInstance,
    /// The vote instance exceeded the 24-bit Graphene vote-id instance range.
    InstanceOutOfRange(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPartCount => f.write_str("BitShares vote id must have form type:instance"),
            Self::EmptyPart => f.write_str("BitShares vote id parts must not be empty"),
            Self::InvalidKind => f.write_str("BitShares vote id type must be numeric"),
            Self::UnknownKind(kind) => write!(f, "unknown BitShares vote id type {kind}"),
            Self::InvalidInstance => f.write_str("BitShares vote id instance must be numeric"),
            Self::InstanceOutOfRange(instance) => {
                write!(f, "BitShares vote id instance {instance} exceeds 24 bits")
            }
        }
    }
}

fn parse_vote_id(value: &str) -> Result<Id, Error> {
    let mut parts = value.split(':');
    let kind = parts.next().ok_or(Error::WrongPartCount)?;
    let instance = parts.next().ok_or(Error::WrongPartCount)?;

    if parts.next().is_some() {
        return Err(Error::WrongPartCount);
    }

    if kind.is_empty() || instance.is_empty() {
        return Err(Error::EmptyPart);
    }

    let kind = kind.parse::<u8>().map_err(|_| Error::InvalidKind)?;
    let kind = Kind::try_from(kind)?;
    let instance = instance
        .parse::<u32>()
        .map_err(|_| Error::InvalidInstance)?;

    if instance > MAX_INSTANCE {
        return Err(Error::InstanceOutOfRange(instance));
    }

    Ok(Id { kind, instance })
}

#[cfg(test)]
mod tests {
    use super::{Error, Id, Kind};

    #[test]
    fn parses_vote_ids_from_graphene_json_strings() {
        let vote: Id = "1:34".parse().expect("valid vote id");

        assert_eq!(vote, Id::new(Kind::Witness, 34));
        assert_eq!(vote.kind(), Kind::Witness);
        assert_eq!(vote.instance(), 34);
        assert_eq!(vote.content(), (34 << 8) | 1);
        assert_eq!(vote.to_string(), "1:34");
    }

    #[test]
    fn parses_all_known_vote_kinds() {
        assert_eq!("0:12".parse::<Id>(), Ok(Id::new(Kind::Committee, 12)));
        assert_eq!("1:12".parse::<Id>(), Ok(Id::new(Kind::Witness, 12)));
        assert_eq!("2:12".parse::<Id>(), Ok(Id::new(Kind::Worker, 12)));
    }

    #[test]
    fn rejects_unknown_vote_kinds() {
        assert_eq!("3:12".parse::<Id>(), Err(Error::UnknownKind(3)));
    }

    #[test]
    fn rejects_malformed_vote_ids() {
        assert_eq!("1".parse::<Id>(), Err(Error::WrongPartCount));
        assert_eq!("1:2:3".parse::<Id>(), Err(Error::WrongPartCount));
        assert_eq!(":2".parse::<Id>(), Err(Error::EmptyPart));
        assert_eq!("witness:2".parse::<Id>(), Err(Error::InvalidKind));
        assert_eq!("1:witness".parse::<Id>(), Err(Error::InvalidInstance));
    }

    #[test]
    fn rejects_instances_outside_the_wire_range() {
        assert_eq!(
            "1:16777216".parse::<Id>(),
            Err(Error::InstanceOutOfRange(16_777_216))
        );
    }
}
