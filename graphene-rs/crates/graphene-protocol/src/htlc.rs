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

#[cfg(test)]
mod tests {
    use super::HtlcHash;

    fn parse_htlc_hash(json: &str) -> Result<HtlcHash, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn htlc_hash_deserializes_known_static_variant_tags() {
        let cases = [
            (r#"[0,"ripemd"]"#, HtlcHash::Ripemd160("ripemd".to_owned())),
            (r#"[1,"sha1"]"#, HtlcHash::Sha1("sha1".to_owned())),
            (r#"[2,"sha256"]"#, HtlcHash::Sha256("sha256".to_owned())),
            (r#"[3,"hash160"]"#, HtlcHash::Hash160("hash160".to_owned())),
        ];

        for (json, expected) in cases {
            assert_eq!(parse_htlc_hash(json).unwrap(), expected);
        }
    }

    #[test]
    fn htlc_hash_reports_unknown_static_variant_tags() {
        let error = parse_htlc_hash(r#"[4,"future"]"#).unwrap_err();

        assert!(
            error.to_string().contains("unknown htlc_hash tag 4"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn htlc_hash_rejects_malformed_static_variant_shapes() {
        let cases = [
            ("[]", "missing tag"),
            ("[0]", "missing hash"),
            ("[0,42]", "wrong hash type"),
            (r#"{"tag":0,"hash":"abc"}"#, "non-sequence input"),
            (r#"[0,"abc","extra"]"#, "trailing sequence element"),
        ];

        for (json, description) in cases {
            assert!(
                parse_htlc_hash(json).is_err(),
                "expected {description} to fail for {json}"
            );
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
