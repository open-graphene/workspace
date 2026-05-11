use crate::RestrictionArgument;

/// HTLC-related committee-tunable chain options.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct HtlcOptions {
    pub max_timeout_secs: u32,
    pub max_preimage_size: u32,
}

/// Custom-authority committee-tunable chain options.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct CustomAuthorityOptions {
    pub max_custom_authority_lifetime_seconds: u32,
    pub max_custom_authorities_per_account: u32,
    pub max_custom_authorities_per_account_op: u32,
    pub max_custom_authority_restrictions: u32,
}

/// Extension fields for mutable chain parameters.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct ChainParameterExtensions {
    pub updatable_htlc_options: Option<HtlcOptions>,
    pub custom_authority_options: Option<CustomAuthorityOptions>,
    pub market_fee_network_percent: Option<u16>,
    pub maker_fee_discount_percent: Option<u16>,
}

/// Mutable committee-controlled Graphene chain parameters.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct ChainParameters {
    pub current_fees: RestrictionArgument,
    pub block_interval: u8,
    pub maintenance_interval: u32,
    pub maintenance_skip_slots: u8,
    pub committee_proposal_review_period: u32,
    pub maximum_transaction_size: u32,
    pub maximum_block_size: u32,
    pub maximum_time_until_expiration: u32,
    pub maximum_proposal_lifetime: u32,
    pub maximum_asset_whitelist_authorities: u8,
    pub maximum_asset_feed_publishers: u8,
    pub maximum_witness_count: u16,
    pub maximum_committee_count: u16,
    pub maximum_authority_membership: u16,
    pub reserve_percent_of_fee: u16,
    pub network_percent_of_fee: u16,
    pub lifetime_referrer_percent_of_fee: u16,
    pub cashback_vesting_period_seconds: u32,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub cashback_vesting_threshold: i64,
    pub count_non_member_votes: bool,
    pub allow_non_member_whitelists: bool,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub witness_pay_per_block: i64,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub worker_budget_per_day: i64,
    pub max_predicate_opcode: u16,
    #[serde(deserialize_with = "crate::i64_from_number_or_string")]
    pub fee_liquidation_threshold: i64,
    pub accounts_per_fee_scale: u16,
    pub account_fee_scale_bitshifts: u8,
    pub max_authority_depth: u8,
    pub extensions: ChainParameterExtensions,
}

/// Graphene genesis-time chain invariants.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct ImmutableChainParameters {
    pub min_committee_member_count: u16,
    pub min_witness_count: u16,
    pub num_special_accounts: u32,
    pub num_special_assets: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_parameters_accepts_string_encoded_share_fields() {
        let parameters: ChainParameters = serde_json::from_str(
            r#"
            {
                "current_fees": {},
                "block_interval": 3,
                "maintenance_interval": 3600,
                "maintenance_skip_slots": 3,
                "committee_proposal_review_period": 1209600,
                "maximum_transaction_size": 2048,
                "maximum_block_size": 2000000,
                "maximum_time_until_expiration": 86400,
                "maximum_proposal_lifetime": 2419200,
                "maximum_asset_whitelist_authorities": 10,
                "maximum_asset_feed_publishers": 10,
                "maximum_witness_count": 1001,
                "maximum_committee_count": 1001,
                "maximum_authority_membership": 10,
                "reserve_percent_of_fee": 2000,
                "network_percent_of_fee": 2000,
                "lifetime_referrer_percent_of_fee": 3000,
                "cashback_vesting_period_seconds": 31536000,
                "cashback_vesting_threshold": "10000000",
                "count_non_member_votes": true,
                "allow_non_member_whitelists": false,
                "witness_pay_per_block": "150000",
                "worker_budget_per_day": "500000000",
                "max_predicate_opcode": 2,
                "fee_liquidation_threshold": "100000000",
                "accounts_per_fee_scale": 1000,
                "account_fee_scale_bitshifts": 4,
                "max_authority_depth": 2,
                "extensions": {}
            }
            "#,
        )
        .expect("chain parameters deserialize string-encoded share fields");

        assert_eq!(parameters.cashback_vesting_threshold, 10_000_000);
        assert_eq!(parameters.witness_pay_per_block, 150_000);
        assert_eq!(parameters.worker_budget_per_day, 500_000_000);
        assert_eq!(parameters.fee_liquidation_threshold, 100_000_000);
    }

    #[test]
    fn chain_parameters_rejects_malformed_string_encoded_share_fields() {
        let malformed = r#"
        {
            "current_fees": {},
            "block_interval": 3,
            "maintenance_interval": 3600,
            "maintenance_skip_slots": 3,
            "committee_proposal_review_period": 1209600,
            "maximum_transaction_size": 2048,
            "maximum_block_size": 2000000,
            "maximum_time_until_expiration": 86400,
            "maximum_proposal_lifetime": 2419200,
            "maximum_asset_whitelist_authorities": 10,
            "maximum_asset_feed_publishers": 10,
            "maximum_witness_count": 1001,
            "maximum_committee_count": 1001,
            "maximum_authority_membership": 10,
            "reserve_percent_of_fee": 2000,
            "network_percent_of_fee": 2000,
            "lifetime_referrer_percent_of_fee": 3000,
            "cashback_vesting_period_seconds": 31536000,
            "cashback_vesting_threshold": "not-a-number",
            "count_non_member_votes": true,
            "allow_non_member_whitelists": false,
            "witness_pay_per_block": "150000",
            "worker_budget_per_day": "500000000",
            "max_predicate_opcode": 2,
            "fee_liquidation_threshold": "100000000",
            "accounts_per_fee_scale": 1000,
            "account_fee_scale_bitshifts": 4,
            "max_authority_depth": 2,
            "extensions": {}
        }
        "#;

        assert!(serde_json::from_str::<ChainParameters>(malformed).is_err());
    }
}
