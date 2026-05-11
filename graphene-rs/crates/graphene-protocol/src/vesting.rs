use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use std::fmt;

/// Linear vesting policy used by legacy balance and vesting-balance objects.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct LinearVestingPolicy {
    pub begin_timestamp: String,
    pub vesting_cliff_seconds: u32,
    pub vesting_duration_seconds: u32,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub begin_balance: i64,
}

/// Coin-days-destroyed vesting policy.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct CddVestingPolicy {
    pub vesting_seconds: u32,
    pub start_claim: String,
    #[serde(deserialize_with = "crate::u128_from_number_or_string")]
    pub coin_seconds_earned: u128,
    pub coin_seconds_earned_last_update: String,
}

/// Instant vesting policy.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct InstantVestingPolicy {}

/// Vesting policy stored as Graphene `static_variant` JSON `[tag, value]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VestingPolicy {
    Linear(LinearVestingPolicy),
    Cdd(CddVestingPolicy),
    Instant(InstantVestingPolicy),
}

impl<'de> Deserialize<'de> for VestingPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VestingPolicyVisitor)
    }
}

struct VestingPolicyVisitor;

impl<'de> Visitor<'de> for VestingPolicyVisitor {
    type Value = VestingPolicy;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Graphene vesting_policy static variant [tag, value]")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tag: u64 = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        match tag {
            0 => {
                let policy = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(VestingPolicy::Linear(policy))
            }
            1 => {
                let policy = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(VestingPolicy::Cdd(policy))
            }
            2 => {
                let policy = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(VestingPolicy::Instant(policy))
            }
            other => Err(de::Error::custom(format!(
                "unknown vesting_policy tag {other}"
            ))),
        }
    }
}
