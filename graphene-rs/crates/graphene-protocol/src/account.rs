use crate::RestrictionArgument;
use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use std::fmt;

/// Account-level mutable options reflected in `account_object::options`.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct AccountOptions<AccountId> {
    pub memo_key: String,
    pub voting_account: AccountId,
    pub num_witness: u16,
    pub num_committee: u16,
    pub votes: Vec<String>,
    pub extensions: Vec<RestrictionArgument>,
}

/// Empty special-authority variant used when an account has normal authorities only.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct NoSpecialAuthority {}

/// Special authority controlled by top holders of an asset.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct TopHoldersSpecialAuthority<AssetId> {
    pub asset: AssetId,
    pub num_top_holders: u8,
}

/// Account special authority stored as Graphene `static_variant` JSON `[tag, value]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpecialAuthority<AssetId> {
    None(NoSpecialAuthority),
    TopHolders(TopHoldersSpecialAuthority<AssetId>),
}

impl<'de, AssetId> Deserialize<'de> for SpecialAuthority<AssetId>
where
    AssetId: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SpecialAuthorityVisitor {
            marker: std::marker::PhantomData,
        })
    }
}

struct SpecialAuthorityVisitor<AssetId> {
    marker: std::marker::PhantomData<AssetId>,
}

impl<'de, AssetId> Visitor<'de> for SpecialAuthorityVisitor<AssetId>
where
    AssetId: Deserialize<'de>,
{
    type Value = SpecialAuthority<AssetId>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Graphene special_authority static variant [tag, value]")
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
                let authority = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(SpecialAuthority::None(authority))
            }
            1 => {
                let authority = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(SpecialAuthority::TopHolders(authority))
            }
            other => Err(de::Error::custom(format!(
                "unknown special_authority tag {other}"
            ))),
        }
    }
}
