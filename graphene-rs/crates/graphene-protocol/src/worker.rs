use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
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

/// Worker type stored as Graphene `static_variant` JSON `[tag, value]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkerType<VestingBalanceId> {
    Refund(RefundWorkerType),
    VestingBalance(VestingBalanceWorkerType<VestingBalanceId>),
    Burn(BurnWorkerType),
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
