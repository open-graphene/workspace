use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;

/// Worker that returns all pay to reserve.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct RefundWorkerType {
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub total_burned: i64,
}

/// Worker that pays into a vesting balance.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct VestingBalanceWorkerType<VestingBalanceId> {
    pub balance: VestingBalanceId,
}

/// Worker that permanently destroys all pay.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct BurnWorkerType {
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub total_burned: i64,
}

/// Worker initializer that returns all pay to reserve.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefundWorkerInitializer {}

impl<'de> Deserialize<'de> for RefundWorkerInitializer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(EmptyRefundWorkerInitializerVisitor)
    }
}

struct EmptyRefundWorkerInitializerVisitor;

impl<'de> Visitor<'de> for EmptyRefundWorkerInitializerVisitor {
    type Value = RefundWorkerInitializer;

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

        Ok(RefundWorkerInitializer {})
    }
}

/// Worker initializer that pays into a vesting balance.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct VestingBalanceWorkerInitializer {
    pub pay_vesting_period_days: u16,
}

/// Worker initializer that permanently destroys all pay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BurnWorkerInitializer {}

impl<'de> Deserialize<'de> for BurnWorkerInitializer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(EmptyBurnWorkerInitializerVisitor)
    }
}

struct EmptyBurnWorkerInitializerVisitor;

impl<'de> Visitor<'de> for EmptyBurnWorkerInitializerVisitor {
    type Value = BurnWorkerInitializer;

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

        Ok(BurnWorkerInitializer {})
    }
}

/// Worker initializer stored as Graphene `static_variant` JSON `[tag, value]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkerInitializer {
    Refund(RefundWorkerInitializer),
    VestingBalance(VestingBalanceWorkerInitializer),
    Burn(BurnWorkerInitializer),
}

/// Worker type stored as Graphene `static_variant` JSON `[tag, value]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkerType<VestingBalanceId> {
    Refund(RefundWorkerType),
    VestingBalance(VestingBalanceWorkerType<VestingBalanceId>),
    Burn(BurnWorkerType),
}

impl<'de> Deserialize<'de> for WorkerInitializer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(WorkerInitializerVisitor)
    }
}

struct WorkerInitializerVisitor;

impl<'de> Visitor<'de> for WorkerInitializerVisitor {
    type Value = WorkerInitializer;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Graphene worker_initializer static variant [tag, value]")
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
                let worker = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                WorkerInitializer::Refund(worker)
            }
            1 => {
                let worker = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                WorkerInitializer::VestingBalance(worker)
            }
            2 => {
                let worker = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                WorkerInitializer::Burn(worker)
            }
            other => {
                return Err(de::Error::custom(format!(
                    "unknown worker_initializer tag {other}"
                )));
            }
        };

        if sequence.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(2, &self));
        }

        Ok(value)
    }
}

impl<'de, VestingBalanceId> Deserialize<'de> for WorkerType<VestingBalanceId>
where
    VestingBalanceId: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(WorkerTypeVisitor {
            marker: std::marker::PhantomData,
        })
    }
}

struct WorkerTypeVisitor<VestingBalanceId> {
    marker: std::marker::PhantomData<VestingBalanceId>,
}

impl<'de, VestingBalanceId> Visitor<'de> for WorkerTypeVisitor<VestingBalanceId>
where
    VestingBalanceId: Deserialize<'de>,
{
    type Value = WorkerType<VestingBalanceId>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Graphene worker_type static variant [tag, value]")
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
                let worker = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(WorkerType::Refund(worker))
            }
            1 => {
                let worker = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(WorkerType::VestingBalance(worker))
            }
            2 => {
                let worker = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(WorkerType::Burn(worker))
            }
            other => Err(de::Error::custom(format!(
                "unknown worker_type tag {other}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_initializer_decodes_static_variants() {
        let refund: WorkerInitializer =
            serde_json::from_str(r#"[0,{}]"#).expect("refund initializer deserializes");
        assert_eq!(
            refund,
            WorkerInitializer::Refund(RefundWorkerInitializer {})
        );

        let vesting: WorkerInitializer =
            serde_json::from_str(r#"[1,{"pay_vesting_period_days":30}]"#)
                .expect("vesting-balance initializer deserializes");
        assert_eq!(
            vesting,
            WorkerInitializer::VestingBalance(VestingBalanceWorkerInitializer {
                pay_vesting_period_days: 30,
            })
        );

        let burn: WorkerInitializer =
            serde_json::from_str(r#"[2,{}]"#).expect("burn initializer deserializes");
        assert_eq!(burn, WorkerInitializer::Burn(BurnWorkerInitializer {}));
    }

    #[test]
    fn worker_initializer_rejects_malformed_static_variants() {
        for malformed in [
            r#"[3,{}]"#,
            r#"[1]"#,
            r#"{"tag":1,"value":{}}"#,
            r#"[1,{"pay_vesting_period_days":"30"}]"#,
            r#"[0,[]]"#,
            r#"[2,[]]"#,
            r#"[0,{},{}]"#,
        ] {
            assert!(
                serde_json::from_str::<WorkerInitializer>(malformed).is_err(),
                "malformed initializer should fail: {malformed}"
            );
        }
    }

    #[test]
    fn existing_worker_type_still_decodes_object_variant() {
        let worker: WorkerType<String> = serde_json::from_str(r#"[1,{"balance":"1.13.7"}]"#)
            .expect("stored vesting-balance worker type deserializes");

        assert_eq!(
            worker,
            WorkerType::VestingBalance(VestingBalanceWorkerType {
                balance: "1.13.7".to_owned(),
            })
        );
    }
}
