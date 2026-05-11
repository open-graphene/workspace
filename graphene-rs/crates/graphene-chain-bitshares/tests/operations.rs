use graphene_chain_bitshares::operations::{
    AccountCreateOperation, AccountUpdateOperation, AssertOperation, AssetClaimFeesOperation,
    AssetCreateOperation, AssetPublishFeedOperation, AssetUpdateBitassetOperation,
    AssetUpdateOperation, BalanceClaimOperation, CommitteeMemberUpdateGlobalParametersOperation,
    CreditOfferAcceptOperation, FillOrderOperation, LimitOrderCreateOperation,
    LimitOrderUpdateOperation, Operation, TransferOperation, VestingBalanceCreateOperation,
    WorkerCreateOperation,
};
use graphene_protocol::{
    HtlcHash, LimitOrderAutoAction, Predicate, VestingPolicyInitializer, WorkerInitializer,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn transfer_payload_json() -> Value {
    json!({
        "fee": { "amount": "10", "asset_id": "1.3.0" },
        "from": "1.2.17",
        "to": "1.2.18",
        "amount": { "amount": 100_i64, "asset_id": "1.3.7" },
        "memo": null,
        "extensions": []
    })
}

fn fill_order_payload_json() -> Value {
    json!({
        "fee": { "amount": 0_i64, "asset_id": "1.3.0" },
        "order_id": "1.7.42",
        "account_id": "1.2.17",
        "pays": { "amount": "5000", "asset_id": "1.3.0" },
        "receives": { "amount": 1250_i64, "asset_id": "1.3.7" },
        "fill_price": {
            "base": { "amount": 4_i64, "asset_id": "1.3.0" },
            "quote": { "amount": 1_i64, "asset_id": "1.3.7" }
        },
        "is_maker": true
    })
}

fn balance_claim_payload_json() -> Value {
    json!({
        "balance_owner_key": "BTS8HF8Mtr9TjW1LxQcyP9KWf3BMaj2PCt6HN9YRzysjmrTjbeiE5",
        "balance_to_claim": "1.15.10747",
        "deposit_to_account": "1.2.90744",
        "fee": { "amount": 0_i64, "asset_id": "1.3.0" },
        "total_claimed": { "amount": 81891883_i64, "asset_id": "1.3.0" }
    })
}

fn authority_payload_json() -> Value {
    json!({
        "weight_threshold": 1_u32,
        "account_auths": [],
        "key_auths": [["BTS8HF8Mtr9TjW1LxQcyP9KWf3BMaj2PCt6HN9YRzysjmrTjbeiE5", 1_u16]],
        "address_auths": []
    })
}

fn account_options_payload_json() -> Value {
    json!({
        "memo_key": "BTS8HF8Mtr9TjW1LxQcyP9KWf3BMaj2PCt6HN9YRzysjmrTjbeiE5",
        "voting_account": "1.2.5",
        "num_witness": 0_u16,
        "num_committee": 0_u16,
        "votes": [],
        "extensions": []
    })
}

fn account_create_payload_json() -> Value {
    json!({
        "fee": { "amount": 123_i64, "asset_id": "1.3.0" },
        "registrar": "1.2.17",
        "referrer": "1.2.18",
        "referrer_percent": 5000_u16,
        "name": "typed-account-fixture",
        "owner": authority_payload_json(),
        "active": authority_payload_json(),
        "options": account_options_payload_json(),
        "extensions": {}
    })
}

fn account_update_payload_json() -> Value {
    json!({
        "fee": { "amount": 124_i64, "asset_id": "1.3.0" },
        "account": "1.2.17",
        "owner": null,
        "active": authority_payload_json(),
        "new_options": account_options_payload_json(),
        "extensions": {}
    })
}

fn core_exchange_rate_json() -> Value {
    json!({
        "base": { "amount": 1_i64, "asset_id": "1.3.0" },
        "quote": { "amount": 1_i64, "asset_id": "1.3.1" }
    })
}

fn asset_options_payload_json() -> Value {
    json!({
        "max_supply": "1000000000000",
        "market_fee_percent": 100_u16,
        "max_market_fee": 1000_i64,
        "issuer_permissions": 79_u16,
        "flags": 0_u16,
        "core_exchange_rate": core_exchange_rate_json(),
        "whitelist_authorities": ["1.2.17"],
        "blacklist_authorities": [],
        "whitelist_markets": ["1.3.0"],
        "blacklist_markets": [],
        "description": "typed asset options fixture",
        "extensions": {
            "reward_percent": 10_u16,
            "whitelist_market_fee_sharing": ["1.2.18"],
            "taker_fee_percent": 25_u16
        }
    })
}

fn bitasset_options_payload_json() -> Value {
    json!({
        "feed_lifetime_sec": 86400_u32,
        "minimum_feeds": 1_u8,
        "force_settlement_delay_sec": 3600_u32,
        "force_settlement_offset_percent": 100_u16,
        "maximum_force_settlement_volume": 2000_u16,
        "short_backing_asset": "1.3.0",
        "extensions": {
            "initial_collateral_ratio": 1750_u16,
            "maintenance_collateral_ratio": 1750_u16,
            "maximum_short_squeeze_ratio": 1100_u16,
            "margin_call_fee_ratio": 10_u16,
            "force_settle_fee_percent": 5_u16,
            "black_swan_response_method": 0_u8
        }
    })
}

fn asset_create_payload_json() -> Value {
    json!({
        "fee": { "amount": 125_i64, "asset_id": "1.3.0" },
        "issuer": "1.2.17",
        "symbol": "TYPED",
        "precision": 5_u8,
        "common_options": asset_options_payload_json(),
        "bitasset_opts": bitasset_options_payload_json(),
        "is_prediction_market": false,
        "extensions": [{}]
    })
}

fn asset_update_payload_json() -> Value {
    json!({
        "fee": { "amount": 126_i64, "asset_id": "1.3.0" },
        "issuer": "1.2.17",
        "asset_to_update": "1.3.9",
        "new_issuer": "1.2.18",
        "new_options": asset_options_payload_json(),
        "extensions": {}
    })
}

fn asset_update_bitasset_payload_json() -> Value {
    json!({
        "fee": { "amount": 127_i64, "asset_id": "1.3.0" },
        "issuer": "1.2.17",
        "asset_to_update": "1.3.9",
        "new_options": bitasset_options_payload_json(),
        "extensions": [{}]
    })
}

fn price_json(base_asset_id: &str, quote_asset_id: &str) -> Value {
    json!({
        "base": { "amount": 3_i64, "asset_id": base_asset_id },
        "quote": { "amount": 2_i64, "asset_id": quote_asset_id }
    })
}

fn price_feed_payload_json() -> Value {
    json!({
        "settlement_price": price_json("1.3.9", "1.3.0"),
        "maintenance_collateral_ratio": 1750_u16,
        "maximum_short_squeeze_ratio": 1100_u16,
        "core_exchange_rate": price_json("1.3.0", "1.3.9")
    })
}

fn asset_publish_feed_payload_json() -> Value {
    json!({
        "fee": { "amount": 128_i64, "asset_id": "1.3.0" },
        "publisher": "1.2.17",
        "asset_id": "1.3.9",
        "feed": price_feed_payload_json(),
        "extensions": { "oracle_note": "preserve raw feed extension" }
    })
}

fn asset_claim_fees_payload_json() -> Value {
    json!({
        "fee": { "amount": 129_i64, "asset_id": "1.3.0" },
        "issuer": "1.2.17",
        "amount_to_claim": { "amount": "98765", "asset_id": "1.3.9" },
        "extensions": { "claim_reason": "preserve raw claim extension" }
    })
}

fn credit_offer_accept_payload_json() -> Value {
    json!({
        "fee": { "amount": 130_i64, "asset_id": "1.3.0" },
        "borrower": "1.2.17",
        "offer_id": "1.21.44",
        "borrow_amount": { "amount": 5000_i64, "asset_id": "1.3.9" },
        "collateral": { "amount": "7500", "asset_id": "1.3.0" },
        "max_fee_rate": 250_u32,
        "min_duration_seconds": 86400_u32,
        "extensions": { "credit_note": "preserve raw credit extension" }
    })
}

fn limit_order_update_payload_json() -> Value {
    json!({
        "fee": { "amount": 131_i64, "asset_id": "1.3.0" },
        "seller": "1.2.17",
        "order": "1.7.42",
        "new_price": price_json("1.3.0", "1.3.9"),
        "delta_amount_to_sell": { "amount": "250", "asset_id": "1.3.0" },
        "new_expiration": null,
        "on_fill": [[0_u64, {
            "fee_asset_id": "1.3.0",
            "spread_percent": 250_u16,
            "size_percent": 5000_u16,
            "expiration_seconds": 3600_u32,
            "repeat": true,
            "extensions": [{ "take_profit_note": "raw nested extension" }]
        }]],
        "extensions": [[0_u64, { "order_update_note": "raw extension list entry" }]]
    })
}

fn unknown_nested_operation_payload_json() -> Value {
    json!({
        "future_field": "nested proposal fallback fixture",
        "nested": { "array": [1_u64, 2_u64, 3_u64], "preserve": true }
    })
}

fn htlc_create_payload_json() -> Value {
    json!({
        "fee": { "amount": 132_i64, "asset_id": "1.3.0" },
        "from": "1.2.17",
        "to": "1.2.18",
        "amount": { "amount": "4567", "asset_id": "1.3.9" },
        "preimage_hash": [2_u64, "00112233445566778899aabbccddeeff00112233"],
        "preimage_size": 20_u16,
        "claim_period_seconds": 3600_u32,
        "extensions": [{ "htlc_note": "M003 approved raw fallback remains raw" }]
    })
}

fn htlc_redeem_payload_json() -> Value {
    json!({
        "fee": { "amount": 133_i64, "asset_id": "1.3.0" },
        "htlc_id": "1.16.42",
        "redeemer": "1.2.18",
        "preimage": [1_u8, 2_u8, 3_u8, 4_u8],
        "extensions": [[0_u64, { "redeem_note": "raw redeem extension" }]]
    })
}

fn htlc_redeemed_payload_json() -> Value {
    json!({
        "fee": { "amount": 0_i64, "asset_id": "1.3.0" },
        "htlc_id": "1.16.42",
        "from": "1.2.17",
        "to": "1.2.18",
        "redeemer": "1.2.18",
        "amount": { "amount": "4567", "asset_id": "1.3.9" },
        "htlc_preimage_hash": [3_u64, "abcdef00112233445566778899aabbccddeeff00"],
        "htlc_preimage_size": 20_u16,
        "preimage": [9_u8, 8_u8, 7_u8, 6_u8]
    })
}

fn htlc_extend_payload_json() -> Value {
    json!({
        "fee": { "amount": 134_i64, "asset_id": "1.3.0" },
        "htlc_id": "1.16.42",
        "update_issuer": "1.2.17",
        "seconds_to_add": 7200_u32,
        "extensions": [[0_u64, { "extend_note": "raw extend extension" }]]
    })
}

fn htlc_refund_payload_json() -> Value {
    json!({
        "fee": { "amount": 0_i64, "asset_id": "1.3.0" },
        "htlc_id": "1.16.42",
        "to": "1.2.17",
        "original_htlc_recipient": "1.2.18",
        "htlc_amount": { "amount": "4567", "asset_id": "1.3.9" },
        "htlc_preimage_hash": [1_u64, "sha1-fixture-hash"],
        "htlc_preimage_size": 20_u16
    })
}

fn chain_parameters_payload_json() -> Value {
    json!({
        "current_fees": { "fee_schedule_note": "raw fee schedule fixture" },
        "block_interval": 3_u8,
        "maintenance_interval": 3600_u32,
        "maintenance_skip_slots": 3_u8,
        "committee_proposal_review_period": 1209600_u32,
        "maximum_transaction_size": 2048_u32,
        "maximum_block_size": 2000000_u32,
        "maximum_time_until_expiration": 86400_u32,
        "maximum_proposal_lifetime": 2419200_u32,
        "maximum_asset_whitelist_authorities": 10_u8,
        "maximum_asset_feed_publishers": 10_u8,
        "maximum_witness_count": 1001_u16,
        "maximum_committee_count": 1001_u16,
        "maximum_authority_membership": 10_u16,
        "reserve_percent_of_fee": 2000_u16,
        "network_percent_of_fee": 2000_u16,
        "lifetime_referrer_percent_of_fee": 3000_u16,
        "cashback_vesting_period_seconds": 31536000_u32,
        "cashback_vesting_threshold": "10000000",
        "count_non_member_votes": true,
        "allow_non_member_whitelists": false,
        "witness_pay_per_block": "150000",
        "worker_budget_per_day": "500000000",
        "max_predicate_opcode": 2_u16,
        "fee_liquidation_threshold": "100000000",
        "accounts_per_fee_scale": 1000_u16,
        "account_fee_scale_bitshifts": 4_u8,
        "max_authority_depth": 2_u8,
        "extensions": {
            "updatable_htlc_options": {
                "max_timeout_secs": 86400_u32,
                "max_preimage_size": 1024_u32
            },
            "custom_authority_options": {
                "max_custom_authority_lifetime_seconds": 2592000_u32,
                "max_custom_authorities_per_account": 16_u32,
                "max_custom_authorities_per_account_op": 4_u32,
                "max_custom_authority_restrictions": 64_u32
            },
            "market_fee_network_percent": 200_u16,
            "maker_fee_discount_percent": 100_u16
        }
    })
}

fn committee_member_update_global_parameters_payload_json() -> Value {
    json!({
        "fee": { "amount": 134_i64, "asset_id": "1.3.0" },
        "new_parameters": chain_parameters_payload_json()
    })
}

fn vesting_balance_create_payload_json(policy: Value) -> Value {
    json!({
        "fee": { "amount": 135_i64, "asset_id": "1.3.0" },
        "creator": "1.2.17",
        "owner": "1.2.18",
        "amount": { "amount": "987654321", "asset_id": "1.3.0" },
        "policy": policy
    })
}

fn linear_vesting_policy_initializer_json() -> Value {
    json!([0_u64, {
        "begin_timestamp": "2025-01-04T05:06:07",
        "vesting_cliff_seconds": 600_u32,
        "vesting_duration_seconds": 3600_u32
    }])
}

fn cdd_vesting_policy_initializer_json() -> Value {
    json!([1_u64, {
        "start_claim": "2025-01-05T06:07:08",
        "vesting_seconds": 7200_u32
    }])
}

fn instant_vesting_policy_initializer_json() -> Value {
    json!([2_u64, {}])
}

fn worker_create_payload_json(initializer: Value) -> Value {
    json!({
        "fee": { "amount": 136_i64, "asset_id": "1.3.0" },
        "owner": "1.2.17",
        "work_begin_date": "2025-02-01T00:00:00",
        "work_end_date": "2025-03-01T00:00:00",
        "daily_pay": "1234567",
        "name": "typed-worker-fixture",
        "url": "https://example.invalid/worker",
        "initializer": initializer
    })
}

fn refund_worker_initializer_json() -> Value {
    json!([0_u64, {}])
}

fn vesting_balance_worker_initializer_json() -> Value {
    json!([1_u64, { "pay_vesting_period_days": 30_u16 }])
}

fn burn_worker_initializer_json() -> Value {
    json!([2_u64, {}])
}

fn assert_payload_json(predicates: Value) -> Value {
    json!({
        "fee": { "amount": 137_i64, "asset_id": "1.3.0" },
        "fee_paying_account": "1.2.17",
        "predicates": predicates,
        "required_auths": ["1.2.17"],
        "extensions": []
    })
}

fn assert_predicates_json() -> Value {
    json!([
        [0_u64, { "account_id": "1.2.17", "name": "typed-account-fixture" }],
        [1_u64, { "asset_id": "1.3.0", "symbol": "BTS" }],
        [2_u64, { "id": "00000001abcdef0123456789abcdef0123456789" }]
    ])
}

fn proposal_create_payload_json() -> Value {
    json!({
        "fee": { "amount": 133_i64, "asset_id": "1.3.0" },
        "fee_paying_account": "1.2.17",
        "expiration_time": "2025-01-03T04:05:06",
        "proposed_ops": [
            { "op": [0_u64, transfer_payload_json()] },
            { "op": [999_u64, unknown_nested_operation_payload_json()] },
            { "op": [49_u64, htlc_create_payload_json()] }
        ],
        "review_period_seconds": 3600_u32,
        "extensions": []
    })
}

#[test]
fn operations_enum_deserializes_typed_htlc_static_variant_tags() {
    let htlc_create: Operation =
        serde_json::from_value(json!([49_u64, htlc_create_payload_json()]))
            .expect("tag 49 htlc_create should deserialize through typed enum");
    assert_eq!(htlc_create.tag(), 49);
    assert!(htlc_create.is_typed());
    match htlc_create {
        Operation::HtlcCreate(operation) => {
            assert_eq!(operation.from.to_string(), "1.2.17");
            assert_eq!(operation.to.to_string(), "1.2.18");
            assert_eq!(operation.amount.amount, 4567);
            assert_eq!(operation.amount.asset_id.to_string(), "1.3.9");
            assert_eq!(
                operation.preimage_hash,
                HtlcHash::Sha256("00112233445566778899aabbccddeeff00112233".to_owned())
            );
            assert_eq!(operation.preimage_size, 20);
            assert_eq!(operation.claim_period_seconds, 3600);
            assert_eq!(
                operation.extensions.0,
                json!([{ "htlc_note": "M003 approved raw fallback remains raw" }])
            );
        }
        other => panic!("unexpected operation variant for tag 49: {other:?}"),
    }

    let htlc_redeem: Operation =
        serde_json::from_value(json!([50_u64, htlc_redeem_payload_json()]))
            .expect("tag 50 htlc_redeem should remain typed");
    assert_eq!(htlc_redeem.tag(), 50);
    assert!(htlc_redeem.is_typed());
    match htlc_redeem {
        Operation::HtlcRedeem(operation) => {
            assert_eq!(operation.htlc_id.to_string(), "1.16.42");
            assert_eq!(operation.redeemer.to_string(), "1.2.18");
            assert_eq!(operation.preimage, vec![1, 2, 3, 4]);
            assert_eq!(
                operation.extensions[0].0,
                json!([0_u64, { "redeem_note": "raw redeem extension" }])
            );
        }
        other => panic!("unexpected operation variant for tag 50: {other:?}"),
    }

    let htlc_redeemed: Operation =
        serde_json::from_value(json!([51_u64, htlc_redeemed_payload_json()]))
            .expect("tag 51 htlc_redeemed should deserialize through typed enum");
    assert_eq!(htlc_redeemed.tag(), 51);
    assert!(htlc_redeemed.is_typed());
    match htlc_redeemed {
        Operation::HtlcRedeemed(operation) => {
            assert_eq!(operation.htlc_id.to_string(), "1.16.42");
            assert_eq!(operation.from.to_string(), "1.2.17");
            assert_eq!(operation.to.to_string(), "1.2.18");
            assert_eq!(operation.redeemer.to_string(), "1.2.18");
            assert_eq!(operation.amount.amount, 4567);
            assert_eq!(operation.amount.asset_id.to_string(), "1.3.9");
            assert_eq!(
                operation.htlc_preimage_hash,
                HtlcHash::Hash160("abcdef00112233445566778899aabbccddeeff00".to_owned())
            );
            assert_eq!(operation.htlc_preimage_size, 20);
            assert_eq!(operation.preimage, vec![9, 8, 7, 6]);
        }
        other => panic!("unexpected operation variant for tag 51: {other:?}"),
    }

    let htlc_extend: Operation =
        serde_json::from_value(json!([52_u64, htlc_extend_payload_json()]))
            .expect("tag 52 htlc_extend should remain typed");
    assert_eq!(htlc_extend.tag(), 52);
    assert!(htlc_extend.is_typed());
    match htlc_extend {
        Operation::HtlcExtend(operation) => {
            assert_eq!(operation.htlc_id.to_string(), "1.16.42");
            assert_eq!(operation.update_issuer.to_string(), "1.2.17");
            assert_eq!(operation.seconds_to_add, 7200);
            assert_eq!(
                operation.extensions[0].0,
                json!([0_u64, { "extend_note": "raw extend extension" }])
            );
        }
        other => panic!("unexpected operation variant for tag 52: {other:?}"),
    }

    let htlc_refund: Operation =
        serde_json::from_value(json!([53_u64, htlc_refund_payload_json()]))
            .expect("tag 53 htlc_refund should deserialize through typed enum");
    assert_eq!(htlc_refund.tag(), 53);
    assert!(htlc_refund.is_typed());
    match htlc_refund {
        Operation::HtlcRefund(operation) => {
            assert_eq!(operation.htlc_id.to_string(), "1.16.42");
            assert_eq!(operation.to.to_string(), "1.2.17");
            assert_eq!(operation.original_htlc_recipient.to_string(), "1.2.18");
            assert_eq!(operation.htlc_amount.amount, 4567);
            assert_eq!(operation.htlc_amount.asset_id.to_string(), "1.3.9");
            assert_eq!(
                operation.htlc_preimage_hash,
                HtlcHash::Sha1("sha1-fixture-hash".to_owned())
            );
            assert_eq!(operation.htlc_preimage_size, 20);
        }
        other => panic!("unexpected operation variant for tag 53: {other:?}"),
    }
}

#[test]
fn operations_enum_deserializes_typed_s04_static_variant_tags() {
    let committee_update: Operation = serde_json::from_value(json!([
        31_u64,
        committee_member_update_global_parameters_payload_json()
    ]))
    .expect(
        "tag 31 committee_member_update_global_parameters should deserialize through typed enum",
    );
    assert_eq!(committee_update.tag(), 31);
    assert!(committee_update.is_typed());
    match committee_update {
        Operation::CommitteeMemberUpdateGlobalParameters(operation) => {
            assert_eq!(operation.fee.amount, 134);
            assert_eq!(operation.new_parameters.block_interval, 3);
            assert_eq!(
                operation.new_parameters.cashback_vesting_threshold,
                10_000_000
            );
            assert_eq!(operation.new_parameters.worker_budget_per_day, 500_000_000);
            assert_eq!(
                operation
                    .new_parameters
                    .extensions
                    .market_fee_network_percent,
                Some(200)
            );
            assert_eq!(
                operation.new_parameters.current_fees.0,
                json!({ "fee_schedule_note": "raw fee schedule fixture" })
            );
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let vesting_balance_create: Operation = serde_json::from_value(json!([
        32_u64,
        vesting_balance_create_payload_json(linear_vesting_policy_initializer_json())
    ]))
    .expect("tag 32 vesting_balance_create should deserialize through typed enum");
    assert_eq!(vesting_balance_create.tag(), 32);
    assert!(vesting_balance_create.is_typed());
    match vesting_balance_create {
        Operation::VestingBalanceCreate(operation) => {
            assert_eq!(operation.creator.to_string(), "1.2.17");
            assert_eq!(operation.amount.amount, 987_654_321);
            match operation.policy {
                VestingPolicyInitializer::Linear(policy) => {
                    assert_eq!(policy.begin_timestamp, "2025-01-04T05:06:07");
                    assert_eq!(policy.vesting_cliff_seconds, 600);
                }
                other => panic!("expected linear vesting initializer: {other:?}"),
            }
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let worker_create: Operation = serde_json::from_value(json!([
        34_u64,
        worker_create_payload_json(vesting_balance_worker_initializer_json())
    ]))
    .expect("tag 34 worker_create should deserialize through typed enum");
    assert_eq!(worker_create.tag(), 34);
    assert!(worker_create.is_typed());
    match worker_create {
        Operation::WorkerCreate(operation) => {
            assert_eq!(operation.owner.to_string(), "1.2.17");
            assert_eq!(operation.daily_pay, 1_234_567);
            assert_eq!(operation.name, "typed-worker-fixture");
            match operation.initializer {
                WorkerInitializer::VestingBalance(initializer) => {
                    assert_eq!(initializer.pay_vesting_period_days, 30);
                }
                other => panic!("expected vesting-balance worker initializer: {other:?}"),
            }
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let assert_operation: Operation = serde_json::from_value(json!([
        36_u64,
        assert_payload_json(assert_predicates_json())
    ]))
    .expect("tag 36 assert should deserialize through typed enum");
    assert_eq!(assert_operation.tag(), 36);
    assert!(assert_operation.is_typed());
    match assert_operation {
        Operation::Assert(operation) => {
            assert_eq!(operation.fee_paying_account.to_string(), "1.2.17");
            assert_eq!(operation.required_auths[0].to_string(), "1.2.17");
            assert!(operation.extensions.is_empty());
            assert_eq!(operation.predicates.len(), 3);
            match &operation.predicates[0] {
                Predicate::AccountNameEqLit(predicate) => {
                    assert_eq!(predicate.account_id.to_string(), "1.2.17");
                    assert_eq!(predicate.name, "typed-account-fixture");
                }
                other => panic!("expected account-name predicate: {other:?}"),
            }
            match &operation.predicates[1] {
                Predicate::AssetSymbolEqLit(predicate) => {
                    assert_eq!(predicate.asset_id.to_string(), "1.3.0");
                    assert_eq!(predicate.symbol, "BTS");
                }
                other => panic!("expected asset-symbol predicate: {other:?}"),
            }
            assert!(matches!(operation.predicates[2], Predicate::BlockId(_)));
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }
}

#[test]
fn operations_deserialize_s04_helper_variant_payloads() {
    let committee_update: CommitteeMemberUpdateGlobalParametersOperation = serde_json::from_value(
        committee_member_update_global_parameters_payload_json(),
    )
    .expect("committee global parameter operation should deserialize typed chain parameters");
    assert_eq!(
        committee_update.new_parameters.witness_pay_per_block,
        150_000
    );
    assert_eq!(
        committee_update.new_parameters.fee_liquidation_threshold,
        100_000_000
    );

    for (policy, expected_tag) in [
        (linear_vesting_policy_initializer_json(), 0_u64),
        (cdd_vesting_policy_initializer_json(), 1_u64),
        (instant_vesting_policy_initializer_json(), 2_u64),
    ] {
        let operation: VestingBalanceCreateOperation =
            serde_json::from_value(vesting_balance_create_payload_json(policy))
                .expect("vesting_balance_create operation should deserialize initializer variant");
        match (expected_tag, operation.policy) {
            (0, VestingPolicyInitializer::Linear(policy)) => {
                assert_eq!(policy.vesting_duration_seconds, 3600);
            }
            (1, VestingPolicyInitializer::Cdd(policy)) => {
                assert_eq!(policy.vesting_seconds, 7200);
            }
            (2, VestingPolicyInitializer::Instant(_)) => {}
            (_, other) => panic!("unexpected vesting initializer variant: {other:?}"),
        }
    }

    for (initializer, expected_tag) in [
        (refund_worker_initializer_json(), 0_u64),
        (vesting_balance_worker_initializer_json(), 1_u64),
        (burn_worker_initializer_json(), 2_u64),
    ] {
        let operation: WorkerCreateOperation =
            serde_json::from_value(worker_create_payload_json(initializer))
                .expect("worker_create operation should deserialize initializer variant");
        match (expected_tag, operation.initializer) {
            (0, WorkerInitializer::Refund(_)) => {}
            (1, WorkerInitializer::VestingBalance(initializer)) => {
                assert_eq!(initializer.pay_vesting_period_days, 30);
            }
            (2, WorkerInitializer::Burn(_)) => {}
            (_, other) => panic!("unexpected worker initializer variant: {other:?}"),
        }
    }

    let assert_operation: AssertOperation =
        serde_json::from_value(assert_payload_json(assert_predicates_json()))
            .expect("assert operation should deserialize all predicate variants");
    assert_eq!(assert_operation.predicates.len(), 3);
    assert!(matches!(
        assert_operation.predicates.as_slice(),
        [
            Predicate::AccountNameEqLit(_),
            Predicate::AssetSymbolEqLit(_),
            Predicate::BlockId(_)
        ]
    ));
}

#[test]
fn operations_enum_rejects_malformed_s04_helper_payloads_instead_of_falling_back() {
    let mut malformed_vesting =
        vesting_balance_create_payload_json(linear_vesting_policy_initializer_json());
    malformed_vesting["policy"] = json!([99_u64, {}]);
    let bad_vesting_tag = serde_json::from_value::<Operation>(json!([32_u64, malformed_vesting]))
        .expect_err("known tag 32 should reject unknown vesting initializer tag");
    assert!(
        bad_vesting_tag
            .to_string()
            .contains("unknown vesting_policy_initializer tag 99"),
        "unexpected error: {bad_vesting_tag}"
    );

    let mut missing_vesting_field =
        vesting_balance_create_payload_json(linear_vesting_policy_initializer_json());
    missing_vesting_field["policy"] = json!([0_u64, {
        "begin_timestamp": "2025-01-04T05:06:07",
        "vesting_cliff_seconds": 600_u32
    }]);
    let missing_vesting_field_error =
        serde_json::from_value::<Operation>(json!([32_u64, missing_vesting_field]))
            .expect_err("known tag 32 should reject missing vesting initializer fields");
    assert!(
        missing_vesting_field_error
            .to_string()
            .contains("vesting_duration_seconds"),
        "unexpected error: {missing_vesting_field_error}"
    );

    let mut non_array_vesting =
        vesting_balance_create_payload_json(linear_vesting_policy_initializer_json());
    non_array_vesting["policy"] = json!({ "tag": 0_u64, "value": {} });
    let non_array_vesting_error =
        serde_json::from_value::<Operation>(json!([32_u64, non_array_vesting]))
            .expect_err("known tag 32 should reject non-array vesting initializer static_variant");
    assert!(
        non_array_vesting_error
            .to_string()
            .contains("vesting_policy_initializer")
            || non_array_vesting_error.to_string().contains("invalid type"),
        "unexpected error: {non_array_vesting_error}"
    );

    let mut malformed_worker = worker_create_payload_json(refund_worker_initializer_json());
    malformed_worker["initializer"] = json!([42_u64, {}]);
    let bad_worker_tag = serde_json::from_value::<Operation>(json!([34_u64, malformed_worker]))
        .expect_err("known tag 34 should reject unknown worker initializer tag");
    assert!(
        bad_worker_tag
            .to_string()
            .contains("unknown worker_initializer tag 42"),
        "unexpected error: {bad_worker_tag}"
    );

    let mut missing_worker_field =
        worker_create_payload_json(vesting_balance_worker_initializer_json());
    missing_worker_field["initializer"] = json!([1_u64, {}]);
    let missing_worker_field_error =
        serde_json::from_value::<Operation>(json!([34_u64, missing_worker_field]))
            .expect_err("known tag 34 should reject missing worker initializer fields");
    assert!(
        missing_worker_field_error
            .to_string()
            .contains("pay_vesting_period_days"),
        "unexpected error: {missing_worker_field_error}"
    );

    let mut non_array_worker = worker_create_payload_json(refund_worker_initializer_json());
    non_array_worker["initializer"] = json!({ "tag": 0_u64, "value": {} });
    let non_array_worker_error =
        serde_json::from_value::<Operation>(json!([34_u64, non_array_worker]))
            .expect_err("known tag 34 should reject non-array worker initializer static_variant");
    assert!(
        non_array_worker_error
            .to_string()
            .contains("worker_initializer")
            || non_array_worker_error.to_string().contains("invalid type"),
        "unexpected error: {non_array_worker_error}"
    );

    let mut bad_predicate_tag = assert_payload_json(assert_predicates_json());
    bad_predicate_tag["predicates"] = json!([[88_u64, {}]]);
    let bad_predicate_tag_error =
        serde_json::from_value::<Operation>(json!([36_u64, bad_predicate_tag]))
            .expect_err("known tag 36 should reject unknown predicate tag");
    assert!(
        bad_predicate_tag_error
            .to_string()
            .contains("unknown predicate tag 88"),
        "unexpected error: {bad_predicate_tag_error}"
    );

    let mut missing_predicate_field = assert_payload_json(assert_predicates_json());
    missing_predicate_field["predicates"] = json!([[0_u64, { "name": "missing account id" }]]);
    let missing_predicate_field_error =
        serde_json::from_value::<Operation>(json!([36_u64, missing_predicate_field]))
            .expect_err("known tag 36 should reject missing predicate fields");
    assert!(
        missing_predicate_field_error
            .to_string()
            .contains("account_id"),
        "unexpected error: {missing_predicate_field_error}"
    );

    let mut non_array_predicate = assert_payload_json(assert_predicates_json());
    non_array_predicate["predicates"] = json!([{ "tag": 0_u64, "value": {} }]);
    let non_array_predicate_error =
        serde_json::from_value::<Operation>(json!([36_u64, non_array_predicate]))
            .expect_err("known tag 36 should reject non-array predicate static_variant");
    assert!(
        non_array_predicate_error.to_string().contains("predicate")
            || non_array_predicate_error
                .to_string()
                .contains("invalid type"),
        "unexpected error: {non_array_predicate_error}"
    );
}

#[test]
fn operation_enum_deserializes_typed_static_variant_tags() {
    let transfer: Operation = serde_json::from_value(json!([0_u64, transfer_payload_json()]))
        .expect("tag 0 transfer operation should deserialize through typed enum");
    assert_eq!(transfer.tag(), 0);
    assert!(transfer.is_typed());
    match transfer {
        Operation::Transfer(operation) => {
            assert_eq!(operation.from.to_string(), "1.2.17");
            assert_eq!(operation.amount.asset_id.to_string(), "1.3.7");
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let fill_order: Operation = serde_json::from_value(json!([4_u64, fill_order_payload_json()]))
        .expect("tag 4 fill_order operation should deserialize through typed enum");
    assert_eq!(fill_order.tag(), 4);
    assert!(fill_order.is_typed());
    assert!(matches!(fill_order, Operation::FillOrder(_)));

    let balance_claim: Operation =
        serde_json::from_value(json!([37_u64, balance_claim_payload_json()]))
            .expect("tag 37 balance_claim operation should deserialize through typed enum");
    assert_eq!(balance_claim.tag(), 37);
    assert!(balance_claim.is_typed());
    assert!(matches!(balance_claim, Operation::BalanceClaim(_)));
}

#[test]
fn operation_enum_deserializes_typed_account_and_asset_static_variant_tags() {
    let account_create: Operation =
        serde_json::from_value(json!([5_u64, account_create_payload_json()]))
            .expect("tag 5 account_create operation should deserialize through typed enum");
    assert_eq!(account_create.tag(), 5);
    assert!(account_create.is_typed());
    match account_create {
        Operation::AccountCreate(operation) => {
            assert_eq!(operation.name, "typed-account-fixture");
            assert_eq!(operation.options.voting_account.to_string(), "1.2.5");
            assert_eq!(operation.extensions.0, json!({}));
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let account_update: Operation =
        serde_json::from_value(json!([6_u64, account_update_payload_json()]))
            .expect("tag 6 account_update operation should deserialize through typed enum");
    assert_eq!(account_update.tag(), 6);
    assert!(account_update.is_typed());
    match account_update {
        Operation::AccountUpdate(operation) => {
            assert_eq!(operation.account.to_string(), "1.2.17");
            assert!(operation.owner.is_none());
            assert_eq!(
                operation
                    .new_options
                    .expect("new_options should deserialize")
                    .memo_key,
                "BTS8HF8Mtr9TjW1LxQcyP9KWf3BMaj2PCt6HN9YRzysjmrTjbeiE5"
            );
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let asset_create: Operation =
        serde_json::from_value(json!([10_u64, asset_create_payload_json()]))
            .expect("tag 10 asset_create operation should deserialize through typed enum");
    assert_eq!(asset_create.tag(), 10);
    assert!(asset_create.is_typed());
    match asset_create {
        Operation::AssetCreate(operation) => {
            assert_eq!(operation.symbol, "TYPED");
            assert_eq!(operation.common_options.max_supply, 1_000_000_000_000);
            assert!(operation.bitasset_opts.is_some());
            assert_eq!(operation.extensions[0].0, json!({}));
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let asset_update: Operation =
        serde_json::from_value(json!([11_u64, asset_update_payload_json()]))
            .expect("tag 11 asset_update operation should deserialize through typed enum");
    assert_eq!(asset_update.tag(), 11);
    assert!(asset_update.is_typed());
    match asset_update {
        Operation::AssetUpdate(operation) => {
            assert_eq!(operation.asset_to_update.to_string(), "1.3.9");
            assert_eq!(
                operation
                    .new_issuer
                    .expect("new issuer should deserialize")
                    .to_string(),
                "1.2.18"
            );
            assert_eq!(operation.new_options.extensions.reward_percent, Some(10));
            assert_eq!(operation.extensions.0, json!({}));
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let asset_update_bitasset: Operation =
        serde_json::from_value(json!([12_u64, asset_update_bitasset_payload_json()]))
            .expect("tag 12 asset_update_bitasset operation should deserialize through typed enum");
    assert_eq!(asset_update_bitasset.tag(), 12);
    assert!(asset_update_bitasset.is_typed());
    match asset_update_bitasset {
        Operation::AssetUpdateBitasset(operation) => {
            assert_eq!(operation.asset_to_update.to_string(), "1.3.9");
            assert_eq!(
                operation.new_options.short_backing_asset.to_string(),
                "1.3.0"
            );
            assert_eq!(operation.extensions[0].0, json!({}));
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }
}

#[test]
fn operation_enum_deserializes_typed_s02_static_variant_tags() {
    let asset_publish_feed: Operation =
        serde_json::from_value(json!([19_u64, asset_publish_feed_payload_json()]))
            .expect("tag 19 asset_publish_feed operation should deserialize through typed enum");
    assert_eq!(asset_publish_feed.tag(), 19);
    assert!(asset_publish_feed.is_typed());
    match asset_publish_feed {
        Operation::AssetPublishFeed(operation) => {
            assert_eq!(operation.publisher.to_string(), "1.2.17");
            assert_eq!(operation.asset_id.to_string(), "1.3.9");
            assert_eq!(operation.feed.maintenance_collateral_ratio, 1750);
            assert_eq!(
                operation.extensions.0,
                json!({ "oracle_note": "preserve raw feed extension" })
            );
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let asset_claim_fees: Operation =
        serde_json::from_value(json!([43_u64, asset_claim_fees_payload_json()]))
            .expect("tag 43 asset_claim_fees operation should deserialize through typed enum");
    assert_eq!(asset_claim_fees.tag(), 43);
    assert!(asset_claim_fees.is_typed());
    match asset_claim_fees {
        Operation::AssetClaimFees(operation) => {
            assert_eq!(operation.issuer.to_string(), "1.2.17");
            assert_eq!(operation.amount_to_claim.amount, 98765);
            assert_eq!(
                operation.extensions.0,
                json!({ "claim_reason": "preserve raw claim extension" })
            );
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let credit_offer_accept: Operation =
        serde_json::from_value(json!([72_u64, credit_offer_accept_payload_json()]))
            .expect("tag 72 credit_offer_accept operation should deserialize through typed enum");
    assert_eq!(credit_offer_accept.tag(), 72);
    assert!(credit_offer_accept.is_typed());
    match credit_offer_accept {
        Operation::CreditOfferAccept(operation) => {
            assert_eq!(operation.borrower.to_string(), "1.2.17");
            assert_eq!(operation.offer_id.to_string(), "1.21.44");
            assert_eq!(operation.max_fee_rate, 250);
            assert_eq!(
                operation.extensions.0,
                json!({ "credit_note": "preserve raw credit extension" })
            );
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }

    let limit_order_update: Operation =
        serde_json::from_value(json!([77_u64, limit_order_update_payload_json()]))
            .expect("tag 77 limit_order_update operation should deserialize through typed enum");
    assert_eq!(limit_order_update.tag(), 77);
    assert!(limit_order_update.is_typed());
    match limit_order_update {
        Operation::LimitOrderUpdate(operation) => {
            assert_eq!(operation.seller.to_string(), "1.2.17");
            assert_eq!(operation.order.to_string(), "1.7.42");
            assert!(operation.new_expiration.is_none());
            assert_eq!(
                operation.extensions[0].0,
                json!([0_u64, { "order_update_note": "raw extension list entry" }])
            );
            let on_fill = operation
                .on_fill
                .expect("on_fill actions should deserialize");
            assert_eq!(on_fill.len(), 1);
            match &on_fill[0] {
                LimitOrderAutoAction::CreateTakeProfitOrder(action) => {
                    assert_eq!(action.fee_asset_id.to_string(), "1.3.0");
                    assert_eq!(action.spread_percent, 250);
                    assert_eq!(action.size_percent, 5000);
                    assert!(action.repeat);
                    assert_eq!(
                        action.extensions[0].0,
                        json!({ "take_profit_note": "raw nested extension" })
                    );
                }
            }
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }
}

#[test]
fn operation_enum_deserializes_proposal_create_nested_htlc_operation_as_typed() {
    let operation: Operation =
        serde_json::from_value(json!([22_u64, proposal_create_payload_json()]))
            .expect("tag 22 proposal_create operation should deserialize through typed enum");
    assert_eq!(operation.tag(), 22);
    assert!(operation.is_typed());

    match operation {
        Operation::ProposalCreate(proposal) => {
            assert_eq!(proposal.fee_paying_account.to_string(), "1.2.17");
            assert_eq!(proposal.review_period_seconds, Some(3600));
            assert!(proposal.extensions.is_empty());
            assert_eq!(proposal.proposed_ops.len(), 3);

            match &proposal.proposed_ops[0].op {
                Operation::Transfer(transfer) => {
                    assert_eq!(transfer.from.to_string(), "1.2.17");
                    assert_eq!(transfer.to.to_string(), "1.2.18");
                    assert_eq!(transfer.amount.asset_id.to_string(), "1.3.7");
                }
                other => panic!("nested known tag 0 should be typed transfer: {other:?}"),
            }
            assert!(proposal.proposed_ops[0].op.is_typed());

            assert_eq!(proposal.proposed_ops[1].op.tag(), 999);
            assert_eq!(
                proposal.proposed_ops[1].op,
                Operation::Unsupported {
                    tag: 999,
                    payload: unknown_nested_operation_payload_json()
                }
            );
            assert!(!proposal.proposed_ops[1].op.is_typed());

            let nested_htlc = &proposal.proposed_ops[2].op;
            assert_eq!(nested_htlc.tag(), 49);
            assert!(nested_htlc.is_typed());
            match nested_htlc {
                Operation::HtlcCreate(operation) => {
                    assert_eq!(operation.from.to_string(), "1.2.17");
                    assert_eq!(operation.to.to_string(), "1.2.18");
                    assert_eq!(operation.amount.amount, 4567);
                    assert_eq!(operation.amount.asset_id.to_string(), "1.3.9");
                    assert_eq!(
                        operation.preimage_hash,
                        HtlcHash::Sha256("00112233445566778899aabbccddeeff00112233".to_owned())
                    );
                    assert_eq!(operation.claim_period_seconds, 3600);
                    assert_eq!(
                        operation.extensions.0,
                        json!([{ "htlc_note": "M003 approved raw fallback remains raw" }])
                    );
                }
                other => panic!("nested known tag 49 should be typed htlc_create: {other:?}"),
            }
        }
        other => panic!("unexpected operation variant: {other:?}"),
    }
}

#[test]
fn operation_enum_rejects_malformed_proposal_nested_wrappers_instead_of_falling_back() {
    let mut missing_op = proposal_create_payload_json();
    missing_op["proposed_ops"] = json!([{ "note": "missing nested op field" }]);
    let missing_op_error = serde_json::from_value::<Operation>(json!([22_u64, missing_op]))
        .expect_err("missing nested op field should reject typed proposal_create payload");
    assert!(
        missing_op_error.to_string().contains("op"),
        "unexpected error: {missing_op_error}"
    );

    let mut malformed_static_variant = proposal_create_payload_json();
    malformed_static_variant["proposed_ops"] = json!([{ "op": [0_u64] }]);
    let malformed_static_variant_error =
        serde_json::from_value::<Operation>(json!([22_u64, malformed_static_variant]))
            .expect_err("malformed nested operation static_variant should reject proposal_create");
    assert!(
        malformed_static_variant_error
            .to_string()
            .contains("two-element"),
        "unexpected error: {malformed_static_variant_error}"
    );
}

#[test]
fn operations_enum_rejects_malformed_known_htlc_payloads_with_clear_errors() {
    let mut unknown_create_hash = htlc_create_payload_json();
    unknown_create_hash["preimage_hash"] = json!([9_u64, "future"]);
    let unknown_create_hash_error =
        serde_json::from_value::<Operation>(json!([49_u64, unknown_create_hash]))
            .expect_err("known tag 49 should reject unknown htlc_hash tags");
    assert!(
        unknown_create_hash_error
            .to_string()
            .contains("unknown htlc_hash tag 9"),
        "unexpected error: {unknown_create_hash_error}"
    );

    let mut missing_claim_period = htlc_create_payload_json();
    missing_claim_period
        .as_object_mut()
        .expect("htlc_create fixture should be an object")
        .remove("claim_period_seconds");
    let missing_claim_period_error =
        serde_json::from_value::<Operation>(json!([49_u64, missing_claim_period]))
            .expect_err("known tag 49 should reject missing claim_period_seconds");
    assert!(
        missing_claim_period_error
            .to_string()
            .contains("claim_period_seconds"),
        "unexpected error: {missing_claim_period_error}"
    );

    let mut missing_preimage_hash = htlc_create_payload_json();
    missing_preimage_hash
        .as_object_mut()
        .expect("htlc_create fixture should be an object")
        .remove("preimage_hash");
    let missing_preimage_hash_error =
        serde_json::from_value::<Operation>(json!([49_u64, missing_preimage_hash]))
            .expect_err("known tag 49 should reject missing preimage_hash");
    assert!(
        missing_preimage_hash_error
            .to_string()
            .contains("preimage_hash"),
        "unexpected error: {missing_preimage_hash_error}"
    );

    let mut unknown_redeemed_hash = htlc_redeemed_payload_json();
    unknown_redeemed_hash["htlc_preimage_hash"] = json!([4_u64, "future"]);
    let unknown_redeemed_hash_error =
        serde_json::from_value::<Operation>(json!([51_u64, unknown_redeemed_hash]))
            .expect_err("known tag 51 should reject unknown htlc_preimage_hash tags");
    assert!(
        unknown_redeemed_hash_error
            .to_string()
            .contains("unknown htlc_hash tag 4")
            || unknown_redeemed_hash_error
                .to_string()
                .contains("htlc_preimage_hash"),
        "unexpected error: {unknown_redeemed_hash_error}"
    );

    let mut malformed_nested_htlc = htlc_create_payload_json();
    malformed_nested_htlc["preimage_hash"] = json!([9_u64, "future"]);
    let mut proposal_with_malformed_htlc = proposal_create_payload_json();
    proposal_with_malformed_htlc["proposed_ops"] =
        json!([{ "op": [49_u64, malformed_nested_htlc] }]);
    let malformed_nested_htlc_error =
        serde_json::from_value::<Operation>(json!([22_u64, proposal_with_malformed_htlc]))
            .expect_err("known proposal tag 22 should reject nested malformed known HTLC payloads");
    assert!(
        malformed_nested_htlc_error
            .to_string()
            .contains("unknown htlc_hash tag 9"),
        "unexpected error: {malformed_nested_htlc_error}"
    );
}

#[test]
fn operation_enum_preserves_unknown_payloads_as_unsupported() {
    let unknown_payload = json!({ "future": "payload", "nested": [1, 2, 3] });
    let unknown: Operation = serde_json::from_value(json!([999_u64, unknown_payload.clone()]))
        .expect("unknown high tag should deserialize as explicit fallback");
    assert_eq!(unknown.tag(), 999);
    assert!(!unknown.is_typed());
    assert_eq!(
        unknown,
        Operation::Unsupported {
            tag: 999,
            payload: unknown_payload
        }
    );
}

#[test]
fn operation_enum_preserves_deferred_blind_confidential_payloads_as_unsupported() {
    for (tag, payload) in [
        (
            39_u16,
            json!({
                "inputs": [
                    {
                        "commitment": "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        "owner": "BTS8HF8Mtr9TjW1LxQcyP9KWf3BMaj2PCt6HN9YRzysjmrTjbeiE5"
                    }
                ],
                "outputs": [
                    {
                        "commitment": "03bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        "range_proof": "synthetic-range-proof",
                        "memo": { "message": "blind transfer deferred fallback" }
                    }
                ],
                "blinding_factor": "000102030405060708090a0b0c0d0e0f",
                "deferred_marker": "blind_transfer_operation"
            }),
        ),
        (
            40_u16,
            json!({
                "fee": { "amount": 0_i64, "asset_id": "1.3.0" },
                "from": "1.2.17",
                "amount": { "amount": "12345", "asset_id": "1.3.9" },
                "outputs": [
                    {
                        "owner": "BTS8HF8Mtr9TjW1LxQcyP9KWf3BMaj2PCt6HN9YRzysjmrTjbeiE5",
                        "stealth_memo": [0_u64, { "nonce": "tag-40", "payload": [1_u8, 2_u8, 3_u8] }]
                    }
                ],
                "deferred_marker": "transfer_to_blind_operation"
            }),
        ),
        (
            41_u16,
            json!({
                "fee": { "amount": 1_i64, "asset_id": "1.3.0" },
                "to": "1.2.18",
                "amount": { "amount": "67890", "asset_id": "1.3.9" },
                "inputs": [
                    {
                        "commitment": "04cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                        "authority": { "weight_threshold": 1_u32, "key_auths": [], "account_auths": [], "address_auths": [] }
                    }
                ],
                "deferred_marker": "transfer_from_blind_operation"
            }),
        ),
    ] {
        let operation: Operation = serde_json::from_value(json!([tag, payload.clone()]))
            .expect("deferred blind/confidential operation should preserve raw payload");
        assert_eq!(operation.tag(), tag);
        assert!(!operation.is_typed());
        assert_eq!(operation, Operation::Unsupported { tag, payload });
    }
}

#[test]
fn operation_enum_rejects_malformed_static_variant_shapes() {
    for malformed in [
        json!({ "tag": 0 }),
        json!([]),
        json!([0]),
        json!([0, {}, null]),
    ] {
        let error = serde_json::from_value::<Operation>(malformed)
            .expect_err("operation static_variant must be a two-element array");
        assert!(
            error.to_string().contains("two-element") || error.to_string().contains("array"),
            "unexpected error: {error}"
        );
    }

    let bad_tag = serde_json::from_value::<Operation>(json!(["0", transfer_payload_json()]))
        .expect_err("operation tag must be numeric");
    assert!(
        bad_tag.to_string().contains("tag"),
        "unexpected error: {bad_tag}"
    );

    let bad_payload = serde_json::from_value::<Operation>(json!([0_u64, { "fee": null }]))
        .expect_err("typed operation payload errors should be reported");
    assert!(
        bad_payload.to_string().contains("invalid type") || bad_payload.to_string().contains("fee"),
        "unexpected error: {bad_payload}"
    );
}

#[test]
fn operations_deserialize_transfer_payload_with_protocol_safe_fields() {
    let operation: TransferOperation = serde_json::from_value(transfer_payload_json())
        .expect("transfer operation should deserialize from Graphene payload JSON");

    assert_eq!(operation.fee.amount, 10);
    assert_eq!(operation.fee.asset_id.space(), 1);
    assert_eq!(operation.fee.asset_id.type_id(), 3);
    assert_eq!(operation.fee.asset_id.instance(), 0);
    assert_eq!(operation.from.to_string(), "1.2.17");
    assert_eq!(operation.to.to_string(), "1.2.18");
    assert_eq!(operation.amount.amount, 100);
    assert_eq!(operation.amount.asset_id.to_string(), "1.3.7");
    assert!(operation.memo.is_none());
    assert!(operation.extensions.is_empty());
}

#[test]
fn operations_deserialize_limit_order_create_payload_with_empty_extensions() {
    let operation: LimitOrderCreateOperation = serde_json::from_value(json!({
        "fee": { "amount": 20_i64, "asset_id": "1.3.0" },
        "seller": "1.2.17",
        "amount_to_sell": { "amount": "100000", "asset_id": "1.3.0" },
        "min_to_receive": { "amount": 25000_i64, "asset_id": "1.3.7" },
        "expiration": "2025-01-02T03:04:05",
        "fill_or_kill": false,
        "extensions": []
    }))
    .expect("limit_order_create operation should deserialize from Graphene payload JSON");

    assert_eq!(operation.seller.space(), 1);
    assert_eq!(operation.seller.type_id(), 2);
    assert_eq!(operation.seller.instance(), 17);
    assert_eq!(operation.amount_to_sell.amount, 100000);
    assert_eq!(operation.amount_to_sell.asset_id.to_string(), "1.3.0");
    assert_eq!(operation.min_to_receive.amount, 25000);
    assert_eq!(operation.min_to_receive.asset_id.to_string(), "1.3.7");
    assert_eq!(operation.expiration, "2025-01-02T03:04:05");
    assert!(!operation.fill_or_kill);
    assert!(operation.extensions.is_empty());
}

#[test]
fn operations_deserialize_fill_order_virtual_payload() {
    let operation: FillOrderOperation = serde_json::from_value(fill_order_payload_json())
        .expect("fill_order operation should deserialize from Graphene payload JSON");

    assert_eq!(operation.order_id.to_string(), "1.7.42");
    assert_eq!(operation.account_id.to_string(), "1.2.17");
    assert_eq!(operation.pays.amount, 5000);
    assert_eq!(operation.pays.asset_id.to_string(), "1.3.0");
    assert_eq!(operation.receives.amount, 1250);
    assert_eq!(operation.receives.asset_id.to_string(), "1.3.7");
    assert_eq!(operation.fill_price.base.amount, 4);
    assert_eq!(operation.fill_price.base.asset_id.to_string(), "1.3.0");
    assert_eq!(operation.fill_price.quote.amount, 1);
    assert_eq!(operation.fill_price.quote.asset_id.to_string(), "1.3.7");
    assert!(operation.is_maker);
}

#[test]
fn operations_deserialize_balance_claim_payload_from_tracked_fixture_shape() {
    let operation: BalanceClaimOperation = serde_json::from_value(balance_claim_payload_json())
        .expect("balance_claim operation should deserialize from inline fixture-shaped JSON");

    assert_eq!(operation.fee.amount, 0);
    assert_eq!(operation.fee.asset_id.to_string(), "1.3.0");
    assert_eq!(operation.deposit_to_account.to_string(), "1.2.90744");
    assert_eq!(operation.balance_to_claim.space(), 1);
    assert_eq!(operation.balance_to_claim.type_id(), 15);
    assert_eq!(operation.balance_to_claim.instance(), 10747);
    assert_eq!(
        operation.balance_owner_key,
        "BTS8HF8Mtr9TjW1LxQcyP9KWf3BMaj2PCt6HN9YRzysjmrTjbeiE5"
    );
    assert_eq!(operation.total_claimed.amount, 81891883);
    assert_eq!(operation.total_claimed.asset_id.to_string(), "1.3.0");
}

#[test]
fn operations_deserialize_account_and_asset_payloads_with_typed_options() {
    let account_create: AccountCreateOperation =
        serde_json::from_value(account_create_payload_json())
            .expect("account_create operation should deserialize typed account_options");
    assert_eq!(account_create.registrar.to_string(), "1.2.17");
    assert_eq!(account_create.referrer_percent, 5000);
    assert_eq!(account_create.owner.weight_threshold, 1);
    assert_eq!(account_create.options.voting_account.to_string(), "1.2.5");
    assert!(account_create.options.extensions.is_empty());
    assert_eq!(account_create.extensions.0, json!({}));

    let account_update: AccountUpdateOperation =
        serde_json::from_value(account_update_payload_json())
            .expect("account_update operation should deserialize typed optional account_options");
    assert_eq!(account_update.account.to_string(), "1.2.17");
    assert!(account_update.owner.is_none());
    assert!(account_update.active.is_some());
    assert_eq!(
        account_update
            .new_options
            .expect("new_options should deserialize")
            .num_committee,
        0
    );
    assert_eq!(account_update.extensions.0, json!({}));

    let asset_create: AssetCreateOperation = serde_json::from_value(asset_create_payload_json())
        .expect("asset_create operation should deserialize typed asset and bitasset options");
    assert_eq!(asset_create.issuer.to_string(), "1.2.17");
    assert_eq!(asset_create.precision, 5);
    assert_eq!(asset_create.common_options.market_fee_percent, 100);
    assert_eq!(
        asset_create
            .bitasset_opts
            .expect("bitasset options should deserialize")
            .extensions
            .initial_collateral_ratio,
        Some(1750)
    );
    assert_eq!(asset_create.extensions[0].0, json!({}));

    let asset_update: AssetUpdateOperation = serde_json::from_value(asset_update_payload_json())
        .expect("asset_update operation should deserialize typed new asset_options");
    assert_eq!(asset_update.asset_to_update.to_string(), "1.3.9");
    assert_eq!(asset_update.new_options.max_market_fee, 1000);
    assert_eq!(
        asset_update.new_options.extensions.taker_fee_percent,
        Some(25)
    );
    assert_eq!(asset_update.extensions.0, json!({}));

    let asset_update_bitasset: AssetUpdateBitassetOperation =
        serde_json::from_value(asset_update_bitasset_payload_json())
            .expect("asset_update_bitasset operation should deserialize typed bitasset_options");
    assert_eq!(asset_update_bitasset.asset_to_update.to_string(), "1.3.9");
    assert_eq!(asset_update_bitasset.new_options.minimum_feeds, 1);
    assert_eq!(
        asset_update_bitasset
            .new_options
            .extensions
            .black_swan_response_method,
        Some(0)
    );
    assert_eq!(asset_update_bitasset.extensions[0].0, json!({}));
}

#[test]
fn operations_deserialize_s02_payloads_with_typed_structs_and_raw_extensions() {
    let asset_publish_feed: AssetPublishFeedOperation =
        serde_json::from_value(asset_publish_feed_payload_json())
            .expect("asset_publish_feed operation should deserialize typed price_feed");
    assert_eq!(asset_publish_feed.fee.amount, 128);
    assert_eq!(asset_publish_feed.publisher.to_string(), "1.2.17");
    assert_eq!(
        asset_publish_feed
            .feed
            .settlement_price
            .base
            .asset_id
            .to_string(),
        "1.3.9"
    );
    assert_eq!(
        asset_publish_feed.extensions.0,
        json!({ "oracle_note": "preserve raw feed extension" })
    );

    let asset_claim_fees: AssetClaimFeesOperation =
        serde_json::from_value(asset_claim_fees_payload_json())
            .expect("asset_claim_fees operation should deserialize typed amount_to_claim");
    assert_eq!(asset_claim_fees.issuer.to_string(), "1.2.17");
    assert_eq!(
        asset_claim_fees.amount_to_claim.asset_id.to_string(),
        "1.3.9"
    );
    assert_eq!(
        asset_claim_fees.extensions.0,
        json!({ "claim_reason": "preserve raw claim extension" })
    );

    let credit_offer_accept: CreditOfferAcceptOperation =
        serde_json::from_value(credit_offer_accept_payload_json())
            .expect("credit_offer_accept operation should deserialize typed credit fields");
    assert_eq!(credit_offer_accept.borrow_amount.amount, 5000);
    assert_eq!(credit_offer_accept.collateral.amount, 7500);
    assert_eq!(credit_offer_accept.min_duration_seconds, 86400);
    assert_eq!(
        credit_offer_accept.extensions.0,
        json!({ "credit_note": "preserve raw credit extension" })
    );

    let limit_order_update: LimitOrderUpdateOperation =
        serde_json::from_value(limit_order_update_payload_json())
            .expect("limit_order_update operation should deserialize typed market actions");
    assert_eq!(limit_order_update.seller.to_string(), "1.2.17");
    assert_eq!(
        limit_order_update
            .new_price
            .expect("new_price should deserialize")
            .quote
            .asset_id
            .to_string(),
        "1.3.9"
    );
    let on_fill = limit_order_update
        .on_fill
        .expect("on_fill should deserialize as optional action vector");
    assert!(matches!(
        on_fill.as_slice(),
        [LimitOrderAutoAction::CreateTakeProfitOrder(_)]
    ));
}

#[test]
fn operations_deserialize_approved_raw_extension_fallback_shape() {
    let operation: TransferOperation = serde_json::from_value(json!({
        "fee": { "amount": 10_i64, "asset_id": "1.3.0" },
        "from": "1.2.17",
        "to": "1.2.18",
        "amount": { "amount": 100_i64, "asset_id": "1.3.0" },
        "memo": null,
        "extensions": [[0_u64, { "policy": "raw-extension-fixture" }]]
    }))
    .expect("approved raw extension fallback should preserve Graphene static_variant JSON");

    assert_eq!(operation.extensions.len(), 1);
    assert_eq!(
        operation.extensions[0].0,
        json!([0_u64, { "policy": "raw-extension-fixture" }])
    );
}

#[test]
fn operations_reject_malformed_generated_payloads() {
    let bad_object_id = serde_json::from_value::<TransferOperation>(json!({
        "fee": { "amount": 10_i64, "asset_id": "1.3.0" },
        "from": "1.2",
        "to": "1.2.18",
        "amount": { "amount": 100_i64, "asset_id": "1.3.0" },
        "memo": null,
        "extensions": []
    }))
    .expect_err("malformed object id should be rejected");
    assert!(
        bad_object_id
            .to_string()
            .contains("object id must have space.type.instance parts"),
        "unexpected error: {bad_object_id}"
    );

    let missing_required_field = serde_json::from_value::<LimitOrderCreateOperation>(json!({
        "fee": { "amount": 20_i64, "asset_id": "1.3.0" },
        "seller": "1.2.17",
        "amount_to_sell": { "amount": 100000_i64, "asset_id": "1.3.0" },
        "expiration": "2025-01-02T03:04:05",
        "fill_or_kill": false,
        "extensions": []
    }))
    .expect_err("missing min_to_receive should be rejected");
    assert!(
        missing_required_field
            .to_string()
            .contains("min_to_receive"),
        "unexpected error: {missing_required_field}"
    );

    let mut account_without_options = account_create_payload_json();
    account_without_options
        .as_object_mut()
        .expect("fixture should be an object")
        .remove("options");
    let missing_account_options =
        serde_json::from_value::<Operation>(json!([5_u64, account_without_options])).expect_err(
            "missing typed account_options should reject tag 5 instead of falling back",
        );
    assert!(
        missing_account_options.to_string().contains("options"),
        "unexpected error: {missing_account_options}"
    );

    let mut asset_without_options = asset_update_payload_json();
    asset_without_options
        .as_object_mut()
        .expect("fixture should be an object")
        .remove("new_options");
    let missing_asset_options =
        serde_json::from_value::<Operation>(json!([11_u64, asset_without_options]))
            .expect_err("missing typed asset_options should reject tag 11 instead of falling back");
    assert!(
        missing_asset_options.to_string().contains("new_options"),
        "unexpected error: {missing_asset_options}"
    );

    let mut limit_order_update_with_unknown_action = limit_order_update_payload_json();
    limit_order_update_with_unknown_action["on_fill"] = json!([[99_u64, {}]]);
    let unknown_auto_action = serde_json::from_value::<Operation>(json!([
        77_u64,
        limit_order_update_with_unknown_action
    ]))
    .expect_err("unknown limit_order_auto_action tag should reject tag 77 payload");
    assert!(
        unknown_auto_action
            .to_string()
            .contains("unknown limit_order_auto_action tag 99"),
        "unexpected error: {unknown_auto_action}"
    );
}

#[derive(Debug)]
struct OperationReportRow {
    operation: String,
    tag: String,
    field: String,
    cpp_type: String,
    source: String,
    classification: String,
    reason: String,
}

impl OperationReportRow {
    fn key(&self) -> (String, String, String, String) {
        (
            self.operation.clone(),
            self.tag.clone(),
            self.field.clone(),
            self.cpp_type.clone(),
        )
    }

    fn detail(&self) -> String {
        format!(
            "operation={} tag={} field={} cpp_type={} source={} classification={} reason={}",
            self.operation,
            self.tag,
            self.field,
            self.cpp_type,
            self.source,
            self.classification,
            self.reason
        )
    }
}

fn strip_markdown_code(value: &str) -> String {
    let trimmed = value.trim();
    trimmed
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or(trimmed)
        .to_string()
}

fn parse_operation_report_rows(report: &str) -> Result<Vec<OperationReportRow>, String> {
    let mut rows = Vec::new();
    let mut in_table = false;

    for (line_index, line) in report.lines().enumerate() {
        let line_number = line_index + 1;
        if line.starts_with("| Operation | Tag | Field | Normalized C++ type |") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if !line.starts_with('|') {
            if !rows.is_empty() {
                break;
            }
            continue;
        }
        if line.starts_with("|---") {
            continue;
        }
        if !line.ends_with('|') {
            return Err(format!(
                "line {line_number}: malformed operation_model_skips.md table row missing trailing pipe: {line}"
            ));
        }

        let cells: Vec<_> = line[1..line.len() - 1].split('|').map(str::trim).collect();
        if cells.len() != 7 {
            return Err(format!(
                "line {line_number}: malformed operation_model_skips.md table row expected 7 cells, found {}: {line}",
                cells.len()
            ));
        }
        if cells.iter().any(|cell| cell.is_empty()) {
            return Err(format!(
                "line {line_number}: malformed operation_model_skips.md table row has an empty required cell: {line}"
            ));
        }

        rows.push(OperationReportRow {
            operation: cells[0].to_string(),
            tag: cells[1].to_string(),
            field: cells[2].to_string(),
            cpp_type: strip_markdown_code(cells[3]),
            source: cells[4].to_string(),
            classification: cells[5].to_string(),
            reason: cells[6].to_string(),
        });
    }

    Ok(rows)
}

fn expected_m003_unsupported_rows() -> BTreeSet<(String, String, String, String)> {
    [
        (
            "transfer_to_blind_operation",
            "39",
            "blinding_factor",
            "blind_factor_type",
        ),
        (
            "transfer_to_blind_operation",
            "39",
            "outputs",
            "vector<blind_output>",
        ),
        (
            "blind_transfer_operation",
            "40",
            "inputs",
            "vector<blind_input>",
        ),
        (
            "blind_transfer_operation",
            "40",
            "outputs",
            "vector<blind_output>",
        ),
        (
            "transfer_from_blind_operation",
            "41",
            "blinding_factor",
            "blind_factor_type",
        ),
        (
            "transfer_from_blind_operation",
            "41",
            "inputs",
            "vector<blind_input>",
        ),
    ]
    .into_iter()
    .map(|(operation, tag, field, cpp_type)| {
        (
            operation.to_string(),
            tag.to_string(),
            field.to_string(),
            cpp_type.to_string(),
        )
    })
    .collect()
}

fn duplicate_unsupported_row_details(rows: &[&OperationReportRow]) -> Vec<String> {
    let mut rows_by_key: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for row in rows {
        rows_by_key.entry(row.key()).or_default().push(*row);
    }

    rows_by_key
        .into_iter()
        .filter(|(_, matching_rows)| matching_rows.len() > 1)
        .map(|((operation, tag, field, cpp_type), matching_rows)| {
            let details = matching_rows
                .iter()
                .map(|row| format!("  - {}", row.detail()))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "duplicate unsupported row key: operation={operation} tag={tag} field={field} cpp_type={cpp_type}\n{details}"
            )
        })
        .collect()
}

#[test]
fn operation_report_only_leaves_m003_deferred_unsupported_rows() {
    let report = include_str!("../src/operation_model_skips.md");
    let rows = parse_operation_report_rows(report).expect("operation report table should parse");
    let unsupported_rows: Vec<_> = rows
        .iter()
        .filter(|row| row.classification == "unsupported")
        .collect();
    let duplicate_rows = duplicate_unsupported_row_details(&unsupported_rows);
    assert!(
        duplicate_rows.is_empty(),
        "operation_model_skips.md contains duplicate unsupported row keys\n{}",
        duplicate_rows.join("\n")
    );

    let actual: BTreeSet<_> = unsupported_rows.iter().map(|row| row.key()).collect();
    let expected = expected_m003_unsupported_rows();

    let unexpected: Vec<_> = actual
        .difference(&expected)
        .map(|key| {
            unsupported_rows
                .iter()
                .find(|row| row.key() == *key)
                .map(|row| row.detail())
                .unwrap_or_else(|| {
                    format!(
                        "operation={} tag={} field={} cpp_type={}",
                        key.0, key.1, key.2, key.3
                    )
                })
        })
        .collect();
    let missing: Vec<_> = expected
        .difference(&actual)
        .map(|(operation, tag, field, cpp_type)| {
            format!("operation={operation} tag={tag} field={field} cpp_type={cpp_type}")
        })
        .collect();

    assert!(
        unexpected.is_empty() && missing.is_empty(),
        "operation_model_skips.md unsupported boundary mismatch\nunexpected unsupported rows:\n{}\nmissing expected M003 rows:\n{}",
        unexpected.join("\n"),
        missing.join("\n")
    );
}

#[test]
fn operation_report_parser_rejects_malformed_table_rows() {
    let malformed_report = "# Operation model coverage report\n\n| Operation | Tag | Field | Normalized C++ type | Source | Classification | Reason |\n|---|---:|---|---|---|---|---|\n| transfer_operation | 0 | extensions | `extensions_type` | source.hpp:1 | unsupported |";
    let error = parse_operation_report_rows(malformed_report)
        .expect_err("malformed report row should fail closed");

    assert!(
        error.contains("malformed operation_model_skips.md table row"),
        "unexpected parser error: {error}"
    );
}

#[test]
fn operation_report_rejects_duplicate_unsupported_row_keys() {
    let duplicated_report = "# Operation model coverage report\n\n| Operation | Tag | Field | Normalized C++ type | Source | Classification | Reason |\n|---|---:|---|---|---|---|---|\n| htlc_create_operation | 49 | preimage_hash | `htlc_hash` | source-a.hpp:60 | unsupported | first duplicate fixture |\n| htlc_create_operation | 49 | preimage_hash | `htlc_hash` | source-b.hpp:61 | unsupported | second duplicate fixture |\n";
    let rows = parse_operation_report_rows(duplicated_report)
        .expect("duplicate report rows should still parse before coverage validation");
    let unsupported_rows: Vec<_> = rows
        .iter()
        .filter(|row| row.classification == "unsupported")
        .collect();
    let duplicate_rows = duplicate_unsupported_row_details(&unsupported_rows);

    assert_eq!(
        duplicate_rows.len(),
        1,
        "expected one duplicate key diagnostic"
    );
    let diagnostic = &duplicate_rows[0];
    assert!(
        diagnostic.contains(
            "duplicate unsupported row key: operation=htlc_create_operation tag=49 field=preimage_hash cpp_type=htlc_hash"
        ),
        "missing duplicate key in diagnostic: {diagnostic}"
    );
    assert!(
        diagnostic.contains("source-a.hpp:60") && diagnostic.contains("source-b.hpp:61"),
        "missing duplicate row details in diagnostic: {diagnostic}"
    );
}

#[test]
fn operations_report_names_raw_fallback_and_unsupported_metadata() {
    let report = include_str!("../src/operation_model_skips.md");

    assert!(report.contains("| transfer_operation | 0 | extensions | `extensions_type` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/transfer.hpp:62 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |"));
    assert!(!report.contains("| account_create_operation | 5 | options | `account_options` |"));
    assert!(report.contains("| account_create_operation | 5 | extensions | `extension<ext>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/account.hpp:113 | approved_raw_fallback | approved raw fallback for operation-scoped extension<ext> payload |"));
    assert!(report.contains("| account_update_operation | 6 | extensions | `extension<ext>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/account.hpp:162 | approved_raw_fallback | approved raw fallback for operation-scoped extension<ext> payload |"));
    assert!(report.contains("| asset_update_operation | 11 | extensions | `extension<ext>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/asset_ops.hpp:377 | approved_raw_fallback | approved raw fallback for operation-scoped extension<ext> payload |"));

    for unsupported_s02_row in [
        "| asset_publish_feed_operation | 19 | feed | `price_feed` |",
        "| asset_claim_fees_operation | 43 | extensions | `extension<additional_options_type>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/asset_ops.hpp:548 | unsupported |",
        "| credit_offer_accept_operation | 72 | extensions | `extension<ext>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/credit_offer.hpp:152 | unsupported |",
        "| limit_order_update_operation | 77 | on_fill | `optional<vector<limit_order_auto_action>>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/market.hpp:129 | unsupported |",
        "| proposal_create_operation | 22 | proposed_ops | `vector<op_wrapper>` |",
        "| htlc_create_operation | 49 | preimage_hash | `htlc_hash` |",
        "| htlc_redeemed_operation | 51 | htlc_preimage_hash | `htlc_hash` |",
        "| htlc_refund_operation | 53 | htlc_preimage_hash | `htlc_hash` |",
    ] {
        assert!(
            !report.contains(unsupported_s02_row),
            "S02/S03 row should no longer be unsupported: {unsupported_s02_row}"
        );
    }

    for unsupported_s04_row in [
        "| committee_member_update_global_parameters_operation | 31 | new_parameters | `chain_parameters` |",
        "| vesting_balance_create_operation | 32 | policy | `vesting_policy_initializer` |",
        "| worker_create_operation | 34 | initializer | `worker_initializer` |",
        "| assert_operation | 36 | predicates | `vector<predicate>` |",
    ] {
        assert!(
            !report.contains(unsupported_s04_row),
            "S04 row should no longer be unsupported: {unsupported_s04_row}"
        );
    }

    assert!(report.contains("| asset_publish_feed_operation | 19 | extensions | `extension<ext>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/asset_ops.hpp:476 | approved_raw_fallback | approved raw fallback for operation-scoped extension<ext> payload |"));
    assert!(report.contains("| asset_claim_fees_operation | 43 | extensions | `extension<additional_options_type>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/asset_ops.hpp:548 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |"));
    assert!(report.contains("| credit_offer_accept_operation | 72 | extensions | `extension<ext>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/credit_offer.hpp:152 | approved_raw_fallback | approved raw fallback for operation-scoped extension<ext> payload |"));
    assert!(report.contains("| limit_order_update_operation | 77 | extensions | `extensions_type` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/market.hpp:131 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |"));
    assert!(report.contains("| proposal_create_operation | 22 | extensions | `extensions_type` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/proposal.hpp:82 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |"));
    assert!(report.contains("| htlc_create_operation | 49 | extensions | `extension<additional_options_type>` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/htlc.hpp:72 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |"));
    assert!(report.contains("| htlc_redeem_operation | 50 | extensions | `extensions_type` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/htlc.hpp:105 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |"));
    assert!(report.contains("| htlc_extend_operation | 52 | extensions | `extensions_type` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/htlc.hpp:168 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |"));
    assert!(report.contains("| assert_operation | 36 | extensions | `extensions_type` | chains/bitshares/bitshares-core/libraries/protocol/include/graphene/protocol/assert.hpp:101 | approved_raw_fallback | approved raw fallback for extension/static-variant payload |"));
}
