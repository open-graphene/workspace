use std::fmt;
use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GrapheneUInt64(u64);

impl GrapheneUInt64 {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for GrapheneUInt64 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl From<GrapheneUInt64> for u64 {
    fn from(value: GrapheneUInt64) -> Self {
        value.0
    }
}

impl From<u64> for GrapheneUInt64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl FromStr for GrapheneUInt64 {
    type Err = std::num::ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse::<u64>().map(Self)
    }
}

impl TryFrom<&str> for GrapheneUInt64 {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for GrapheneUInt64 {
    type Error = std::num::ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Serialize for GrapheneUInt64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

impl<'de> Deserialize<'de> for GrapheneUInt64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = GrapheneUInt64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a u64 number or decimal string")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(GrapheneUInt64(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GrapheneInt64(i64);

impl GrapheneInt64 {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl fmt::Display for GrapheneInt64 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl From<GrapheneInt64> for i64 {
    fn from(value: GrapheneInt64) -> Self {
        value.0
    }
}

impl From<i64> for GrapheneInt64 {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl FromStr for GrapheneInt64 {
    type Err = std::num::ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse::<i64>().map(Self)
    }
}

impl TryFrom<&str> for GrapheneInt64 {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for GrapheneInt64 {
    type Error = std::num::ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Serialize for GrapheneInt64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.0)
    }
}

impl<'de> Deserialize<'de> for GrapheneInt64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = GrapheneInt64;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an i64 number or decimal string")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(GrapheneInt64(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i64::try_from(value).map(GrapheneInt64).map_err(E::custom)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Graphene wire-format timestamp.
///
/// Graphene APIs serialize `fc::time_point_sec` values as UTC timestamps without
/// an explicit timezone suffix, for example `2026-05-13T14:44:57`. This is not
/// RFC 3339 `date-time`, so generated chain bindings use this scalar instead of
/// `chrono::DateTime<Utc>`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GrapheneTimePointSec(NaiveDateTime);

impl GrapheneTimePointSec {
    pub const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S";

    pub fn new(value: NaiveDateTime) -> Self {
        Self(value)
    }

    pub fn naive_utc(self) -> NaiveDateTime {
        self.0
    }
}

impl fmt::Display for GrapheneTimePointSec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.format(Self::FORMAT))
    }
}

impl FromStr for GrapheneTimePointSec {
    type Err = chrono::ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        NaiveDateTime::parse_from_str(value, Self::FORMAT).map(Self)
    }
}

impl TryFrom<&str> for GrapheneTimePointSec {
    type Error = chrono::ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for GrapheneTimePointSec {
    type Error = chrono::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<GrapheneTimePointSec> for String {
    fn from(value: GrapheneTimePointSec) -> Self {
        value.to_string()
    }
}

impl Serialize for GrapheneTimePointSec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GrapheneTimePointSec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&value, Self::FORMAT)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
