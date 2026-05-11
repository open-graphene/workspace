use serde::de::{self, Deserializer, Visitor};
use std::fmt;

/// Deserialize a signed integer from either a JSON number or a decimal string.
pub fn i64_from_number_or_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(I64Visitor)
}

/// Deserialize an unsigned integer from either a JSON number or a decimal string.
pub fn u64_from_number_or_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(U64Visitor)
}

/// Deserialize a 128-bit unsigned integer from either a JSON number or a decimal string.
pub fn u128_from_number_or_string<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(U128Visitor)
}

struct I64Visitor;
struct U64Visitor;
struct U128Visitor;

impl<'de> Visitor<'de> for I64Visitor {
    type Value = i64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an i64 JSON number or decimal string")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(value)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        i64::try_from(value).map_err(|_| E::custom(format!("{value} is out of range for i64")))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse::<i64>().map_err(E::custom)
    }
}

impl<'de> Visitor<'de> for U64Visitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a u64 JSON number or decimal string")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u64::try_from(value).map_err(|_| E::custom(format!("{value} is out of range for u64")))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(value)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse::<u64>().map_err(E::custom)
    }
}

impl<'de> Visitor<'de> for U128Visitor {
    type Value = u128;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a u128 JSON number or decimal string")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u128::try_from(value).map_err(|_| E::custom(format!("{value} is out of range for u128")))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(u128::from(value))
    }

    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E> {
        Ok(value)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value.parse::<u128>().map_err(E::custom)
    }
}
