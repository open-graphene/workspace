use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use std::fmt;

use crate::RestrictionArgument;

/// Action that creates a take-profit order when a limit order is filled.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct CreateTakeProfitOrderAction<AssetId> {
    pub fee_asset_id: AssetId,
    pub spread_percent: u16,
    pub size_percent: u16,
    pub expiration_seconds: u32,
    pub repeat: bool,
    pub extensions: Vec<RestrictionArgument>,
}

/// Automatic action attached to a limit order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LimitOrderAutoAction<AssetId> {
    CreateTakeProfitOrder(CreateTakeProfitOrderAction<AssetId>),
}

impl<'de, AssetId> Deserialize<'de> for LimitOrderAutoAction<AssetId>
where
    AssetId: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(LimitOrderAutoActionVisitor {
            marker: std::marker::PhantomData,
        })
    }
}

struct LimitOrderAutoActionVisitor<AssetId> {
    marker: std::marker::PhantomData<AssetId>,
}

impl<'de, AssetId> Visitor<'de> for LimitOrderAutoActionVisitor<AssetId>
where
    AssetId: Deserialize<'de>,
{
    type Value = LimitOrderAutoAction<AssetId>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Graphene limit_order_auto_action static variant [tag, value]")
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
                let action = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(LimitOrderAutoAction::CreateTakeProfitOrder(action))
            }
            other => Err(de::Error::custom(format!(
                "unknown limit_order_auto_action tag {other}"
            ))),
        }
    }
}
