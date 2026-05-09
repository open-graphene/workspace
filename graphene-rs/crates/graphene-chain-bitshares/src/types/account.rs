graphene_protocol::define_object_id_type! {
    chain: "BitShares",
    chain_article: "a",
    /// A BitShares account object identifier (`1.2.x`).
    id: "account",
    type_id: 2,
    instance_doc: "Returns the account object instance component.",
    wrong_type_doc: "The object id is not in the BitShares account object range (`1.2.x`).",
    wrong_type_message: "object id {actual} is not a BitShares account id",
}

/// A minimal BitShares account object as returned by the database API.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct Object {
    /// Account object id.
    pub id: Id,
    /// Human-readable account name.
    pub name: String,
    /// Account that registered this account.
    pub registrar: Id,
    /// Account credited as this account's referrer.
    pub referrer: Id,
    /// Account credited as this account's lifetime referrer.
    pub lifetime_referrer: Id,
    /// Statistics object for this account.
    pub statistics: crate::types::account_statistics::Id,
    /// Owner authority controlling account ownership changes.
    pub owner: Authority,
    /// Active authority controlling ordinary account operations.
    pub active: Authority,
    /// Account options and voting preferences.
    pub options: Options,
}

/// BitShares authority structure used by account owner and active authorities.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct Authority {
    /// Total weight required for the authority to be satisfied.
    pub weight_threshold: u32,
    /// Account authorities and their weights.
    pub account_auths: Vec<(Id, u16)>,
    /// Public-key authorities and their weights.
    pub key_auths: Vec<(crate::types::public_key::PublicKey, u16)>,
    /// Address authorities and their weights.
    pub address_auths: Vec<(crate::types::address::Address, u16)>,
}

/// BitShares account options as returned inside an account object.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct Options {
    /// Public memo key in the chain's string format.
    pub memo_key: crate::types::public_key::PublicKey,
    /// Account used as voting proxy.
    pub voting_account: Id,
    /// Requested number of witnesses to vote for.
    pub num_witness: u16,
    /// Requested number of committee members to vote for.
    pub num_committee: u16,
    /// Vote ids selected by this account.
    pub votes: Vec<crate::types::vote::Id>,
    /// Account option extensions. Currently only empty extension arrays are modeled.
    pub extensions: Vec<Extension>,
}

/// Placeholder for BitShares account option extensions.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub enum Extension {}

#[cfg(test)]
mod tests {
    use super::{Authority, Error, Id, Object, Options};
    use crate::types::{account_statistics, address, public_key, vote};
    use graphene_protocol::{ObjectId, ObjectIdParseError};

    #[test]
    fn constructs_account_ids_from_instances() {
        let account = Id::new(345);

        assert_eq!(account.object_id(), ObjectId::new(1, 2, 345));
        assert_eq!(ObjectId::from(account), ObjectId::new(1, 2, 345));
        assert_eq!(Id::try_from(ObjectId::new(1, 2, 345)), Ok(account));
    }

    #[test]
    fn rejects_non_account_object_ids() {
        assert_eq!(
            "1.3.345".parse::<Id>(),
            Err(Error::WrongType {
                actual: ObjectId::new(1, 3, 345),
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
    fn deserializes_minimal_account_object_from_bitshares_rpc_json() {
        let json = r#"
        {
            "id": "1.2.3",
            "membership_expiration_date": "2106-02-07T06:28:15",
            "registrar": "1.2.3",
            "referrer": "1.2.3",
            "lifetime_referrer": "1.2.3",
            "network_fee_percentage": 0,
            "lifetime_referrer_fee_percentage": 10000,
            "referrer_rewards_percentage": 0,
            "name": "null-account",
            "owner": { "weight_threshold": 1, "account_auths": [], "key_auths": [], "address_auths": [] },
            "active": { "weight_threshold": 1, "account_auths": [], "key_auths": [], "address_auths": [] },
            "options": {
                "memo_key": "BTS1111111111111111111111111111111114T1Anm",
                "voting_account": "1.2.5",
                "num_witness": 0,
                "num_committee": 0,
                "votes": [],
                "extensions": []
            },
            "num_committee_voted": 5136,
            "statistics": "2.6.3",
            "whitelisting_accounts": ["1.2.125573"],
            "blacklisting_accounts": ["1.2.979278"],
            "whitelisted_accounts": [],
            "blacklisted_accounts": [],
            "owner_special_authority": [0, {}],
            "active_special_authority": [0, {}],
            "top_n_control_flags": 0,
            "creation_block_num": 0,
            "creation_time": "2015-10-13T13:00:00"
        }
        "#;

        let account: Object =
            serde_json::from_str(json).expect("account object should deserialize");

        assert_eq!(account.id, Id::new(3));
        assert_eq!(account.name, "null-account");
        assert_eq!(account.registrar, Id::new(3));
        assert_eq!(account.referrer, Id::new(3));
        assert_eq!(account.lifetime_referrer, Id::new(3));
        assert_eq!(account.statistics, account_statistics::Id::new(3));
        assert_eq!(
            account.options,
            Options {
                memo_key: "BTS1111111111111111111111111111111114T1Anm"
                    .parse::<public_key::PublicKey>()
                    .expect("valid memo key"),
                voting_account: Id::new(5),
                num_witness: 0,
                num_committee: 0,
                votes: Vec::new(),
                extensions: Vec::new(),
            }
        );
        assert_eq!(
            account.owner,
            Authority {
                weight_threshold: 1,
                account_auths: Vec::new(),
                key_auths: Vec::new(),
                address_auths: Vec::new(),
            }
        );
        assert_eq!(
            account.active,
            Authority {
                weight_threshold: 1,
                account_auths: Vec::new(),
                key_auths: Vec::new(),
                address_auths: Vec::new(),
            }
        );
    }

    #[test]
    fn rejects_non_account_statistics_object_ids() {
        let json = r#"
        {
            "id": "1.2.3",
            "registrar": "1.2.3",
            "referrer": "1.2.3",
            "lifetime_referrer": "1.2.3",
            "name": "bad-statistics-account",
            "owner": { "weight_threshold": 1, "account_auths": [], "key_auths": [], "address_auths": [] },
            "active": { "weight_threshold": 1, "account_auths": [], "key_auths": [], "address_auths": [] },
            "options": {
                "memo_key": "BTS1111111111111111111111111111111114T1Anm",
                "voting_account": "1.2.5",
                "num_witness": 0,
                "num_committee": 0,
                "votes": [],
                "extensions": []
            },
            "statistics": "2.5.3"
        }
        "#;

        let error =
            serde_json::from_str::<Object>(json).expect_err("wrong statistics id should fail");

        assert!(error
            .to_string()
            .contains("BitShares account statistics id"));
    }

    #[test]
    fn deserializes_authority_with_account_key_and_address_auths() {
        let json = r#"
        {
            "weight_threshold": 2,
            "account_auths": [["1.2.10", 1]],
            "key_auths": [["BTS1111111111111111111111111111111114T1Anm", 1]],
            "address_auths": [["BTS1111111111111111111111111111111114T1Anm", 1]]
        }
        "#;

        let authority: Authority =
            serde_json::from_str(json).expect("authority should deserialize");

        assert_eq!(
            authority,
            Authority {
                weight_threshold: 2,
                account_auths: vec![(Id::new(10), 1)],
                key_auths: vec![(
                    "BTS1111111111111111111111111111111114T1Anm"
                        .parse::<public_key::PublicKey>()
                        .expect("valid authority key"),
                    1,
                )],
                address_auths: vec![(
                    "BTS1111111111111111111111111111111114T1Anm"
                        .parse::<address::Address>()
                        .expect("valid authority address"),
                    1,
                )],
            }
        );
    }

    #[test]
    fn rejects_public_keys_with_the_wrong_chain_prefix() {
        let json = r#"
        {
            "weight_threshold": 1,
            "account_auths": [],
            "key_auths": [["R2S1111111111111111111111111111111114T1Anm", 1]],
            "address_auths": []
        }
        "#;

        let error = serde_json::from_str::<Authority>(json).expect_err("wrong prefix should fail");

        assert!(error.to_string().contains("BitShares public key"));
    }

    #[test]
    fn deserializes_options_with_vote_ids() {
        let json = r#"
        {
            "memo_key": "BTS1111111111111111111111111111111114T1Anm",
            "voting_account": "1.2.5",
            "num_witness": 1,
            "num_committee": 1,
            "votes": ["0:12", "1:34", "2:56"],
            "extensions": []
        }
        "#;

        let options: Options = serde_json::from_str(json).expect("options should deserialize");

        assert_eq!(
            options.votes,
            vec![
                vote::Id::new(vote::Kind::Committee, 12),
                vote::Id::new(vote::Kind::Witness, 34),
                vote::Id::new(vote::Kind::Worker, 56),
            ]
        );
    }

    #[test]
    fn rejects_addresses_with_the_wrong_chain_prefix() {
        let json = r#"
        {
            "weight_threshold": 1,
            "account_auths": [],
            "key_auths": [],
            "address_auths": [["R2S1111111111111111111111111111111114T1Anm", 1]]
        }
        "#;

        let error = serde_json::from_str::<Authority>(json).expect_err("wrong prefix should fail");

        assert!(error.to_string().contains("BitShares address"));
    }
}
