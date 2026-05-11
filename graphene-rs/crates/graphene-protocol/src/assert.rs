use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};
use std::fmt;

/// Predicate that checks whether an account object's name equals a literal string.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct AccountNameEqLitPredicate<AccountId> {
    pub account_id: AccountId,
    pub name: String,
}

/// Predicate that checks whether an asset object's symbol equals a literal string.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct AssetSymbolEqLitPredicate<AssetId> {
    pub asset_id: AssetId,
    pub symbol: String,
}

/// Predicate that checks whether a block id appears in recent chain history.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct BlockIdPredicate {
    pub id: String,
}

/// Assert-operation predicate stored as Graphene `static_variant` JSON `[tag, value]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Predicate<AccountId, AssetId> {
    AccountNameEqLit(AccountNameEqLitPredicate<AccountId>),
    AssetSymbolEqLit(AssetSymbolEqLitPredicate<AssetId>),
    BlockId(BlockIdPredicate),
}

impl<'de, AccountId, AssetId> Deserialize<'de> for Predicate<AccountId, AssetId>
where
    AccountId: Deserialize<'de>,
    AssetId: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(PredicateVisitor {
            marker: std::marker::PhantomData,
        })
    }
}

struct PredicateVisitor<AccountId, AssetId> {
    marker: std::marker::PhantomData<(AccountId, AssetId)>,
}

impl<'de, AccountId, AssetId> Visitor<'de> for PredicateVisitor<AccountId, AssetId>
where
    AccountId: Deserialize<'de>,
    AssetId: Deserialize<'de>,
{
    type Value = Predicate<AccountId, AssetId>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Graphene assert predicate static variant [tag, value]")
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
                let predicate = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Predicate::AccountNameEqLit(predicate)
            }
            1 => {
                let predicate = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Predicate::AssetSymbolEqLit(predicate)
            }
            2 => {
                let predicate = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Predicate::BlockId(predicate)
            }
            other => return Err(de::Error::custom(format!("unknown predicate tag {other}"))),
        };

        if sequence.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(2, &self));
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type StringPredicate = Predicate<String, String>;

    #[test]
    fn assert_predicate_decodes_static_variants() {
        let account: StringPredicate =
            serde_json::from_str(r#"[0,{"account_id":"1.2.5","name":"committee-account"}]"#)
                .expect("account-name predicate deserializes");
        assert_eq!(
            account,
            Predicate::AccountNameEqLit(AccountNameEqLitPredicate {
                account_id: "1.2.5".to_owned(),
                name: "committee-account".to_owned(),
            })
        );

        let asset: StringPredicate =
            serde_json::from_str(r#"[1,{"asset_id":"1.3.0","symbol":"BTS"}]"#)
                .expect("asset-symbol predicate deserializes");
        assert_eq!(
            asset,
            Predicate::AssetSymbolEqLit(AssetSymbolEqLitPredicate {
                asset_id: "1.3.0".to_owned(),
                symbol: "BTS".to_owned(),
            })
        );

        let block: StringPredicate =
            serde_json::from_str(r#"[2,{"id":"00000001abcdef0123456789abcdef0123456789"}]"#)
                .expect("block-id predicate deserializes");
        assert_eq!(
            block,
            Predicate::BlockId(BlockIdPredicate {
                id: "00000001abcdef0123456789abcdef0123456789".to_owned(),
            })
        );
    }

    #[test]
    fn assert_predicate_rejects_malformed_static_variants() {
        for malformed in [
            r#"[3,{}]"#,
            r#"[0]"#,
            r#"{"tag":0,"value":{}}"#,
            r#"[0,{"account_id":5,"name":"alice"}]"#,
            r#"[1,{"asset_id":"1.3.0","symbol":7}]"#,
            r#"[2,[]]"#,
            r#"[2,{"id":"abc"},{}]"#,
        ] {
            assert!(
                serde_json::from_str::<StringPredicate>(malformed).is_err(),
                "malformed predicate should fail: {malformed}"
            );
        }
    }
}
