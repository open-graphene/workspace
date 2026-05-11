use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use std::fmt;

use crate::RestrictionArgument;

/// HTLC transfer details stored inside an active HTLC object.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct HtlcTransfer<AccountId, AssetId> {
    pub from: AccountId,
    pub to: AccountId,
    pub amount: i64,
    pub asset_id: AssetId,
}

/// HTLC preimage hash as Graphene `static_variant` JSON `[tag, hash]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtlcHash {
    Ripemd160(String),
    Sha1(String),
    Sha256(String),
    Hash160(String),
}

impl<'de> Deserialize<'de> for HtlcHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(HtlcHashVisitor)
    }
}

struct HtlcHashVisitor;

impl<'de> Visitor<'de> for HtlcHashVisitor {
    type Value = HtlcHash;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Graphene htlc_hash static variant [tag, hash]")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tag: u64 = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let hash: String = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;

        match tag {
            0 => Ok(HtlcHash::Ripemd160(hash)),
            1 => Ok(HtlcHash::Sha1(hash)),
            2 => Ok(HtlcHash::Sha256(hash)),
            3 => Ok(HtlcHash::Hash160(hash)),
            other => Err(de::Error::custom(format!("unknown htlc_hash tag {other}"))),
        }
    }
}

/// Hash-lock condition for an HTLC.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct HtlcHashLock {
    pub preimage_hash: HtlcHash,
    pub preimage_size: u16,
}

/// Time-lock condition for an HTLC.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct HtlcTimeLock {
    pub expiration: String,
}

/// Full HTLC condition info.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct HtlcConditions {
    pub hash_lock: HtlcHashLock,
    pub time_lock: HtlcTimeLock,
}

/// Encrypted memo data as exposed by Graphene JSON.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct MemoData {
    pub from: String,
    pub to: String,
    pub nonce: u64,
    pub message: RestrictionArgument,
}
