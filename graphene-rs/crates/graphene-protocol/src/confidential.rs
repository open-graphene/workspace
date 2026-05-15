//! Confidential transfer protocol primitives shared by generated operation models.
//!
//! These types model the JSON shape exposed by Graphene confidential/blind transfer operations.
//! They intentionally do not validate cryptographic commitments, range proofs, keys, or blinding
//! factors; validation belongs to signing/proof-generation layers, not the serde model boundary.

/// Hex/string encoded ECC blinding factor.
pub type BlindFactor = String;

/// Hex/string encoded Pedersen commitment.
pub type Commitment = String;

/// Graphene public key string.
pub type PublicKey = String;

/// Range proof bytes as represented by FC JSON byte arrays.
pub type RangeProof = Vec<u8>;

/// Input consumed by confidential transfer operations.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct BlindInput {
    pub commitment: Commitment,
    pub owner: crate::Authority<crate::ObjectId>,
}

/// Optional stealth memo attached to a confidential output.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct StealthConfirmation {
    pub one_time_key: PublicKey,
    pub to: Option<PublicKey>,
    pub encrypted_memo: Vec<u8>,
}

/// Output produced by confidential transfer operations.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct BlindOutput {
    pub commitment: Commitment,
    pub range_proof: RangeProof,
    pub owner: crate::Authority<crate::ObjectId>,
    pub stealth_memo: Option<StealthConfirmation>,
}

#[cfg(test)]
mod tests {
    use super::{BlindInput, BlindOutput, StealthConfirmation};
    use serde_json::json;

    fn authority_json() -> serde_json::Value {
        json!({
            "weight_threshold": 1_u32,
            "account_auths": [["1.2.17", 1_u16]],
            "key_auths": [],
            "address_auths": []
        })
    }

    #[test]
    fn blind_input_deserializes_commitment_and_authority() {
        let input: BlindInput = serde_json::from_value(json!({
            "commitment": "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "owner": authority_json()
        }))
        .expect("blind input should deserialize");

        assert_eq!(
            input.commitment,
            "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(input.owner.weight_threshold, 1);
        assert_eq!(input.owner.account_auths[0].0.to_string(), "1.2.17");
    }

    #[test]
    fn stealth_confirmation_deserializes_optional_to_and_encrypted_memo() {
        let memo: StealthConfirmation = serde_json::from_value(json!({
            "one_time_key": "BTS1111111111111111111111111111111114T1Anm",
            "to": null,
            "encrypted_memo": [9_u8, 8_u8, 7_u8]
        }))
        .expect("stealth confirmation should deserialize");

        assert_eq!(
            memo.one_time_key,
            "BTS1111111111111111111111111111111114T1Anm"
        );
        assert_eq!(memo.to, None);
        assert_eq!(memo.encrypted_memo, vec![9, 8, 7]);
    }

    #[test]
    fn blind_output_deserializes_range_proof_owner_and_optional_stealth_memo() {
        let output: BlindOutput = serde_json::from_value(json!({
            "commitment": "03bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "range_proof": [1_u8, 2_u8, 3_u8, 4_u8],
            "owner": authority_json(),
            "stealth_memo": {
                "one_time_key": "BTS1111111111111111111111111111111114T1Anm",
                "to": "BTS2222222222222222222222222222222222WMRva",
                "encrypted_memo": [5_u8, 6_u8]
            }
        }))
        .expect("blind output should deserialize");

        assert_eq!(output.range_proof, vec![1, 2, 3, 4]);
        assert_eq!(output.owner.weight_threshold, 1);
        let memo = output.stealth_memo.expect("stealth memo should be present");
        assert_eq!(
            memo.to.as_deref(),
            Some("BTS2222222222222222222222222222222222WMRva")
        );
        assert_eq!(memo.encrypted_memo, vec![5, 6]);
    }
}
