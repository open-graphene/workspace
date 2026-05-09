graphene_protocol::define_object_id_type! {
    chain: "BitShares",
    chain_article: "a",
    /// A BitShares asset object identifier (`1.3.x`).
    id: "asset",
    type_id: 3,
    instance_doc: "Returns the asset object instance component.",
    wrong_type_doc: "The object id is not in the BitShares asset object range (`1.3.x`).",
    wrong_type_message: "object id {actual} is not a BitShares asset id",
}

/// A minimal BitShares asset object as returned by the database API.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct Object {
    /// Asset object id.
    pub id: Id,
    /// Human-readable asset symbol, for example `BTS`.
    pub symbol: String,
    /// Decimal precision used by this asset.
    pub precision: u8,
    /// Account that issued the asset.
    pub issuer: crate::types::account::Id,
}

#[cfg(test)]
mod tests {
    use super::{Error, Id, Object};
    use crate::types::account;
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_asset_ids_from_instances() {
        let asset = Id::new(7);

        assert_eq!(asset.object_id(), ObjectId::new(1, 3, 7));
        assert_eq!(ObjectId::from(asset), ObjectId::new(1, 3, 7));
        assert_eq!(Id::try_from(ObjectId::new(1, 3, 7)), Ok(asset));
    }

    #[test]
    fn rejects_non_asset_object_ids() {
        assert_eq!(
            "1.2.7".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(1, 2, 7),
            })
        );
    }

    #[test]
    fn propagates_object_id_parse_errors() {
        assert_eq!(
            "not-an-id".parse::<Id>(),
            Err(Error::Parse(ObjectIdParseError::WrongPartCount))
        );
    }

    #[test]
    fn deserializes_minimal_asset_object_from_bitshares_rpc_json() {
        let json = r#"
        {
            "id": "1.3.0",
            "symbol": "BTS",
            "precision": 5,
            "issuer": "1.2.3",
            "options": {
                "max_supply": "360057050210207",
                "market_fee_percent": 0,
                "max_market_fee": "1000000000000000",
                "issuer_permissions": 0,
                "flags": 0,
                "core_exchange_rate": {
                    "base": { "amount": 1, "asset_id": "1.3.0" },
                    "quote": { "amount": 1, "asset_id": "1.3.0" }
                },
                "whitelist_authorities": [],
                "blacklist_authorities": [],
                "whitelist_markets": [],
                "blacklist_markets": [],
                "description": "",
                "extensions": {}
            },
            "dynamic_asset_data_id": "2.3.0",
            "creation_block_num": 0,
            "creation_time": "2015-10-13T13:00:00",
            "total_in_collateral": "9775381180349"
        }
        "#;

        let asset: Object = serde_json::from_str(json).expect("asset object should deserialize");

        assert_eq!(asset.id, Id::new(0));
        assert_eq!(asset.symbol, "BTS");
        assert_eq!(asset.precision, 5);
        assert_eq!(asset.issuer, account::Id::new(3));
    }
}
