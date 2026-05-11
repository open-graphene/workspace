use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
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

/// Linear vesting policy initializer used by vesting-balance create operations.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct LinearVestingPolicyInitializer {
    pub begin_timestamp: String,
    pub vesting_cliff_seconds: u32,
    pub vesting_duration_seconds: u32,
}

/// Coin-days-destroyed vesting policy initializer used by vesting-balance create operations.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct CddVestingPolicyInitializer {
    pub start_claim: String,
    pub vesting_seconds: u32,
}

/// Instant vesting policy initializer used by vesting-balance create operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstantVestingPolicyInitializer {}

impl<'de> Deserialize<'de> for InstantVestingPolicyInitializer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(EmptyVestingInitializerVisitor)
    }
}

struct EmptyVestingInitializerVisitor;

impl<'de> Visitor<'de> for EmptyVestingInitializerVisitor {
    type Value = InstantVestingPolicyInitializer;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an empty object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        if let Some(key) = map.next_key::<String>()? {
            return Err(de::Error::unknown_field(&key, &[]));
        }

        Ok(InstantVestingPolicyInitializer {})
    }
}

/// Vesting policy initializer stored as Graphene `static_variant` JSON `[tag, value]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VestingPolicyInitializer {
    Linear(LinearVestingPolicyInitializer),
    Cdd(CddVestingPolicyInitializer),
    Instant(InstantVestingPolicyInitializer),
}

/// Vesting policy stored as Graphene `static_variant` JSON `[tag, value]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VestingPolicy {
    Linear(LinearVestingPolicy),
    Cdd(CddVestingPolicy),
    Instant(InstantVestingPolicy),
}

impl<'de> Deserialize<'de> for VestingPolicyInitializer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(VestingPolicyInitializerVisitor)
    }
}

struct VestingPolicyInitializerVisitor;

impl<'de> Visitor<'de> for VestingPolicyInitializerVisitor {
    type Value = VestingPolicyInitializer;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Graphene vesting_policy_initializer static variant [tag, value]")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tag: u64 = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        let value = match tag {
            0 => {
                let policy = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                VestingPolicyInitializer::Linear(policy)
            }
            1 => {
                let policy = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                VestingPolicyInitializer::Cdd(policy)
            }
            2 => {
                let policy = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                VestingPolicyInitializer::Instant(policy)
            }
            other => {
                return Err(de::Error::custom(format!(
                    "unknown vesting_policy_initializer tag {other}"
                )));
            }
        };

        if sequence.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(2, &self));
        }

        Ok(value)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vesting_policy_initializer_decodes_static_variants() {
        let linear: VestingPolicyInitializer = serde_json::from_str(
            r#"[0,{"begin_timestamp":"2024-01-01T00:00:00","vesting_cliff_seconds":60,"vesting_duration_seconds":120}]"#,
        )
        .expect("linear initializer deserializes");
        assert_eq!(
            linear,
            VestingPolicyInitializer::Linear(LinearVestingPolicyInitializer {
                begin_timestamp: "2024-01-01T00:00:00".to_owned(),
                vesting_cliff_seconds: 60,
                vesting_duration_seconds: 120,
            })
        );

        let cdd: VestingPolicyInitializer = serde_json::from_str(
            r#"[1,{"start_claim":"2024-02-01T00:00:00","vesting_seconds":3600}]"#,
        )
        .expect("cdd initializer deserializes");
        assert_eq!(
            cdd,
            VestingPolicyInitializer::Cdd(CddVestingPolicyInitializer {
                start_claim: "2024-02-01T00:00:00".to_owned(),
                vesting_seconds: 3600,
            })
        );

        let instant: VestingPolicyInitializer =
            serde_json::from_str(r#"[2,{}]"#).expect("instant initializer deserializes");
        assert_eq!(
            instant,
            VestingPolicyInitializer::Instant(InstantVestingPolicyInitializer {})
        );
    }

    #[test]
    fn vesting_policy_initializer_rejects_malformed_static_variants() {
        for malformed in [
            r#"[3,{}]"#,
            r#"[0]"#,
            r#"{"tag":0,"value":{}}"#,
            r#"[0,[]]"#,
            r#"[2,{} ,{}]"#,
            r#"[2,[]]"#,
        ] {
            assert!(
                serde_json::from_str::<VestingPolicyInitializer>(malformed).is_err(),
                "malformed initializer should fail: {malformed}"
            );
        }
    }

    #[test]
    fn existing_vesting_policy_still_decodes_object_variant() {
        let policy: VestingPolicy = serde_json::from_str(
            r#"[1,{"vesting_seconds":3600,"start_claim":"2024-02-01T00:00:00","coin_seconds_earned":"42","coin_seconds_earned_last_update":"2024-02-02T00:00:00"}]"#,
        )
        .expect("stored cdd vesting policy deserializes");

        assert!(matches!(policy, VestingPolicy::Cdd(_)));
    }
}
