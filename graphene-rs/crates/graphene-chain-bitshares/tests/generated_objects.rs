use graphene_chain_bitshares::types::{
    account, account_balance, account_history, account_statistics, asset, asset_bitasset_data,
    asset_dynamic_data, balance, base, blinded_balance, block_summary, buyback, call_order,
    chain_property, collateral_bid, committee_member, credit_deal, credit_deal_summary,
    credit_offer, custom_authority, dynamic_global_property, fba_accumulator, force_settlement,
    global_property, htlc, limit_order, liquidity_pool, null, reserved0, samet_fund,
    special_authority, ticket, vesting_balance, withdraw_permission, witness, witness_schedule,
    worker,
};
use serde_json::json;

#[test]
fn deserializes_marker_objects() {
    let null_object: null::Object = serde_json::from_value(json!({ "id": "1.0.0" }))
        .expect("null marker object should deserialize from Graphene JSON");
    let base_object: base::Object = serde_json::from_value(json!({ "id": "1.1.0" }))
        .expect("base marker object should deserialize from Graphene JSON");
    let reserved_object: reserved0::Object = serde_json::from_value(json!({ "id": "2.2.0" }))
        .expect("reserved0 marker object should deserialize from Graphene JSON");

    assert_eq!(null_object.id.to_string(), "1.0.0");
    assert_eq!(base_object.id.to_string(), "1.1.0");
    assert_eq!(reserved_object.id.to_string(), "2.2.0");
}

#[test]
fn deserializes_account_balance_object() {
    let object: account_balance::Object = serde_json::from_value(json!({
        "id": "2.5.42",
        "owner": "1.2.7",
        "asset_type": "1.3.0",
        "balance": 123456789_i64,
        "maintenance_flag": false
    }))
    .expect("account_balance object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.5.42");
    assert_eq!(object.id.instance(), 42);
    assert_eq!(object.owner.to_string(), "1.2.7");
    assert_eq!(object.asset_type.to_string(), "1.3.0");
    assert_eq!(object.balance, 123456789);
    assert!(!object.maintenance_flag);
}

#[test]
fn deserializes_account_history_object() {
    let object: account_history::Object = serde_json::from_value(json!({
        "id": "2.9.100",
        "account": "1.2.17",
        "operation_id": "1.11.55",
        "sequence": 7_u64,
        "next": "2.9.99"
    }))
    .expect("account_history object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.9.100");
    assert_eq!(object.account.to_string(), "1.2.17");
    assert_eq!(object.operation_id.to_string(), "1.11.55");
    assert_eq!(object.sequence, 7);
    assert_eq!(object.next.to_string(), "2.9.99");
}

#[test]
fn deserializes_account_statistics_object() {
    let object: account_statistics::Object = serde_json::from_value(json!({
        "id": "2.6.17",
        "owner": "1.2.17",
        "name": "alice",
        "most_recent_op": "2.9.100",
        "total_ops": 10_u64,
        "removed_ops": 1_u64,
        "total_core_in_orders": 2_i64,
        "total_core_inactive": 3_i64,
        "total_core_pob": 4_i64,
        "total_core_pol": 5_i64,
        "total_pob_value": 6_i64,
        "total_pol_value": 7_i64,
        "core_in_balance": 8_i64,
        "has_cashback_vb": true,
        "is_voting": true,
        "last_vote_time": "2024-01-02T03:04:05",
        "vp_all": 11_u64,
        "vp_active": 12_u64,
        "vp_committee": 13_u64,
        "vp_witness": 14_u64,
        "vp_worker": 15_u64,
        "vote_tally_time": "2024-01-02T04:04:05",
        "lifetime_fees_paid": 16_i64,
        "pending_fees": 17_i64,
        "pending_vested_fees": 18_i64
    }))
    .expect("account_statistics object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.6.17");
    assert_eq!(object.owner.to_string(), "1.2.17");
    assert_eq!(object.name, "alice");
    assert_eq!(object.most_recent_op.to_string(), "2.9.100");
    assert_eq!(object.total_ops, 10);
    assert_eq!(object.removed_ops, 1);
    assert_eq!(object.total_core_in_orders, 2);
    assert_eq!(object.total_core_inactive, 3);
    assert_eq!(object.total_core_pob, 4);
    assert_eq!(object.total_core_pol, 5);
    assert_eq!(object.total_pob_value, 6);
    assert_eq!(object.total_pol_value, 7);
    assert_eq!(object.core_in_balance, 8);
    assert!(object.has_cashback_vb);
    assert!(object.is_voting);
    assert_eq!(object.last_vote_time, "2024-01-02T03:04:05");
    assert_eq!(object.vp_all, 11);
    assert_eq!(object.vp_active, 12);
    assert_eq!(object.vp_committee, 13);
    assert_eq!(object.vp_witness, 14);
    assert_eq!(object.vp_worker, 15);
    assert_eq!(object.vote_tally_time, "2024-01-02T04:04:05");
    assert_eq!(object.lifetime_fees_paid, 16);
    assert_eq!(object.pending_fees, 17);
    assert_eq!(object.pending_vested_fees, 18);
}

#[test]
fn deserializes_account_object_with_options_and_special_authorities() {
    let key = "BTS1111111111111111111111111111111114T1Anm";
    let object: account::Object = serde_json::from_value(json!({
        "id": "1.2.17",
        "membership_expiration_date": "1969-12-31T23:59:59",
        "registrar": "1.2.1",
        "referrer": "1.2.2",
        "lifetime_referrer": "1.2.3",
        "network_fee_percentage": 2000_u16,
        "lifetime_referrer_fee_percentage": 3000_u16,
        "referrer_rewards_percentage": 4000_u16,
        "name": "alice",
        "owner": {
            "weight_threshold": 1_u32,
            "account_auths": [["1.2.1", 1_u16]],
            "key_auths": [[key, 1_u16]],
            "address_auths": []
        },
        "active": {
            "weight_threshold": 1_u32,
            "account_auths": [],
            "key_auths": [[key, 1_u16]],
            "address_auths": []
        },
        "options": {
            "memo_key": key,
            "voting_account": "1.2.5",
            "num_witness": 21_u16,
            "num_committee": 11_u16,
            "votes": ["0:5", "1:6", "2:7"],
            "extensions": []
        },
        "num_committee_voted": 11_u16,
        "statistics": "2.6.17",
        "whitelisting_accounts": ["1.2.8"],
        "blacklisting_accounts": ["1.2.9"],
        "whitelisted_accounts": ["1.2.10"],
        "blacklisted_accounts": ["1.2.11"],
        "cashback_vb": "1.13.4",
        "owner_special_authority": [0, {}],
        "active_special_authority": [1, {
            "asset": "1.3.0",
            "num_top_holders": 5_u8
        }],
        "top_n_control_flags": 2_u8,
        "allowed_assets": ["1.3.0", "1.3.7"],
        "creation_block_num": 123_u32,
        "creation_time": "2024-01-02T03:04:05"
    }))
    .expect("account object should deserialize with options and special authorities");

    assert_eq!(object.id.to_string(), "1.2.17");
    assert_eq!(object.registrar.to_string(), "1.2.1");
    assert_eq!(object.name, "alice");
    assert_eq!(object.owner.account_auths[0].0.to_string(), "1.2.1");
    assert_eq!(object.active.key_auths[0].0, key);
    assert_eq!(object.options.memo_key, key);
    assert_eq!(object.options.voting_account.to_string(), "1.2.5");
    assert_eq!(object.options.votes, vec!["0:5", "1:6", "2:7"]);
    assert_eq!(object.statistics.to_string(), "2.6.17");
    assert_eq!(object.cashback_vb.unwrap().to_string(), "1.13.4");
    assert!(matches!(
        object.owner_special_authority,
        graphene_protocol::SpecialAuthority::None(_)
    ));
    match object.active_special_authority {
        graphene_protocol::SpecialAuthority::TopHolders(authority) => {
            assert_eq!(authority.asset.to_string(), "1.3.0");
            assert_eq!(authority.num_top_holders, 5);
        }
        graphene_protocol::SpecialAuthority::None(_) => {
            panic!("expected top holders special authority")
        }
    }
    assert_eq!(object.allowed_assets.unwrap()[1].to_string(), "1.3.7");
    assert_eq!(object.creation_block_num, 123);
    assert_eq!(object.creation_time, "2024-01-02T03:04:05");
}

#[test]
fn deserializes_asset_dynamic_data_object() {
    let object: asset_dynamic_data::Object = serde_json::from_value(json!({
        "id": "2.3.0",
        "current_supply": 1_000_000_i64,
        "confidential_supply": 0_i64,
        "accumulated_fees": 10_i64,
        "accumulated_collateral_fees": 20_i64,
        "fee_pool": 30_i64
    }))
    .expect("asset_dynamic_data object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.3.0");
    assert_eq!(object.current_supply, 1_000_000);
    assert_eq!(object.confidential_supply, 0);
    assert_eq!(object.accumulated_fees, 10);
    assert_eq!(object.accumulated_collateral_fees, 20);
    assert_eq!(object.fee_pool, 30);
}

#[test]
fn deserializes_asset_object_with_asset_options() {
    let object: asset::Object = serde_json::from_value(json!({
        "id": "1.3.7",
        "symbol": "USD",
        "precision": 4_u8,
        "issuer": "1.2.17",
        "options": {
            "max_supply": 1_000_000_000_i64,
            "market_fee_percent": 20_u16,
            "max_market_fee": 10_000_i64,
            "issuer_permissions": 79_u16,
            "flags": 1_u16,
            "core_exchange_rate": {
                "base": { "amount": 1_i64, "asset_id": "1.3.0" },
                "quote": { "amount": 2_i64, "asset_id": "1.3.7" }
            },
            "whitelist_authorities": ["1.2.17"],
            "blacklist_authorities": ["1.2.18"],
            "whitelist_markets": ["1.3.0"],
            "blacklist_markets": ["1.3.9"],
            "description": "United States dollar",
            "extensions": {
                "reward_percent": 100_u16,
                "whitelist_market_fee_sharing": ["1.2.19"],
                "taker_fee_percent": 30_u16
            }
        },
        "dynamic_asset_data_id": "2.3.7",
        "bitasset_data_id": "2.4.7",
        "buyback_account": "1.2.20",
        "for_liquidity_pool": "1.19.3",
        "creation_block_num": 42_u32,
        "creation_time": "2024-01-02T03:04:05"
    }))
    .expect("asset object should deserialize with asset options");

    assert_eq!(object.id.to_string(), "1.3.7");
    assert_eq!(object.symbol, "USD");
    assert_eq!(object.precision, 4);
    assert_eq!(object.issuer.to_string(), "1.2.17");
    assert_eq!(object.options.max_supply, 1_000_000_000);
    assert_eq!(
        object.options.core_exchange_rate.base.asset_id.to_string(),
        "1.3.0"
    );
    assert_eq!(
        object.options.whitelist_authorities[0].to_string(),
        "1.2.17"
    );
    assert_eq!(object.options.blacklist_markets[0].to_string(), "1.3.9");
    assert_eq!(object.options.extensions.reward_percent, Some(100));
    assert_eq!(
        object
            .options
            .extensions
            .whitelist_market_fee_sharing
            .unwrap()[0]
            .to_string(),
        "1.2.19"
    );
    assert_eq!(object.options.extensions.taker_fee_percent, Some(30));
    assert_eq!(object.dynamic_asset_data_id.to_string(), "2.3.7");
    assert_eq!(object.bitasset_data_id.unwrap().to_string(), "2.4.7");
    assert_eq!(object.buyback_account.unwrap().to_string(), "1.2.20");
    assert_eq!(object.for_liquidity_pool.unwrap().to_string(), "1.19.3");
    assert_eq!(object.creation_block_num, 42);
    assert_eq!(object.creation_time, "2024-01-02T03:04:05");
}

#[test]
fn deserializes_asset_bitasset_data_object_with_feeds() {
    let feed = json!({
        "settlement_price": {
            "base": { "amount": 3_i64, "asset_id": "1.3.7" },
            "quote": { "amount": 1_i64, "asset_id": "1.3.0" }
        },
        "maintenance_collateral_ratio": 1750_u16,
        "maximum_short_squeeze_ratio": 1100_u16,
        "core_exchange_rate": {
            "base": { "amount": 1_i64, "asset_id": "1.3.0" },
            "quote": { "amount": 3_i64, "asset_id": "1.3.7" }
        },
        "initial_collateral_ratio": 2000_u16
    });

    let object: asset_bitasset_data::Object = serde_json::from_value(json!({
        "id": "2.4.7",
        "asset_id": "1.3.7",
        "feeds": [["1.2.17", ["2024-01-02T03:04:05", feed.clone()]]],
        "median_feed": feed.clone(),
        "current_feed": feed,
        "current_feed_publication_time": "2024-01-02T03:04:05",
        "current_maintenance_collateralization": {
            "base": { "amount": 175_i64, "asset_id": "1.3.0" },
            "quote": { "amount": 100_i64, "asset_id": "1.3.7" }
        },
        "current_initial_collateralization": {
            "base": { "amount": 200_i64, "asset_id": "1.3.0" },
            "quote": { "amount": 100_i64, "asset_id": "1.3.7" }
        },
        "options": {
            "feed_lifetime_sec": 86400_u32,
            "minimum_feeds": 1_u8,
            "force_settlement_delay_sec": 3600_u32,
            "force_settlement_offset_percent": 100_u16,
            "maximum_force_settlement_volume": 2000_u16,
            "short_backing_asset": "1.3.0",
            "extensions": {
                "initial_collateral_ratio": 2000_u16,
                "maintenance_collateral_ratio": 1750_u16,
                "maximum_short_squeeze_ratio": 1100_u16,
                "margin_call_fee_ratio": 50_u16,
                "force_settle_fee_percent": 25_u16,
                "black_swan_response_method": 1_u8
            }
        },
        "force_settled_volume": 123_i64,
        "is_prediction_market": false,
        "settlement_price": {
            "base": { "amount": 0_i64, "asset_id": "1.3.7" },
            "quote": { "amount": 0_i64, "asset_id": "1.3.0" }
        },
        "settlement_fund": 456_i64,
        "individual_settlement_debt": 7_i64,
        "individual_settlement_fund": 8_i64,
        "asset_cer_updated": true,
        "feed_cer_updated": false
    }))
    .expect("asset_bitasset_data object should deserialize with price feeds");

    assert_eq!(object.id.to_string(), "2.4.7");
    assert_eq!(object.asset_id.to_string(), "1.3.7");
    assert_eq!(object.feeds[0].0.to_string(), "1.2.17");
    assert_eq!(object.feeds[0].1.0, "2024-01-02T03:04:05");
    assert_eq!(object.feeds[0].1.1.initial_collateral_ratio, 2000);
    assert_eq!(object.median_feed.maintenance_collateral_ratio, 1750);
    assert_eq!(
        object
            .current_feed
            .core_exchange_rate
            .quote
            .asset_id
            .to_string(),
        "1.3.7"
    );
    assert_eq!(object.options.short_backing_asset.to_string(), "1.3.0");
    assert_eq!(object.options.extensions.margin_call_fee_ratio, Some(50));
    assert_eq!(object.force_settled_volume, 123);
    assert!(!object.is_prediction_market);
    assert_eq!(object.settlement_fund, 456);
    assert!(object.asset_cer_updated);
    assert!(!object.feed_cer_updated);
}

#[test]
fn deserializes_balance_object_with_linear_vesting_policy() {
    let owner = "BTSFN9r6VYzBK8EKtMewfNbfiGCr56pHDBFi";
    let object: balance::Object = serde_json::from_value(json!({
        "id": "1.15.3",
        "owner": owner,
        "balance": {
            "amount": 1000_i64,
            "asset_id": "1.3.0"
        },
        "vesting_policy": {
            "begin_timestamp": "2024-01-01T00:00:00",
            "vesting_cliff_seconds": 3600_u32,
            "vesting_duration_seconds": 86400_u32,
            "begin_balance": 1000_i64
        },
        "last_claim_date": "2024-01-02T00:00:00"
    }))
    .expect("balance object should deserialize with linear vesting policy");

    assert_eq!(object.id.to_string(), "1.15.3");
    assert_eq!(object.owner, owner);
    assert_eq!(object.balance.amount, 1000);
    assert_eq!(object.balance.asset_id.to_string(), "1.3.0");
    let vesting_policy = object
        .vesting_policy
        .expect("linear vesting policy should be present");
    assert_eq!(vesting_policy.begin_timestamp, "2024-01-01T00:00:00");
    assert_eq!(vesting_policy.vesting_cliff_seconds, 3600);
    assert_eq!(vesting_policy.vesting_duration_seconds, 86400);
    assert_eq!(vesting_policy.begin_balance, 1000);
    assert_eq!(object.last_claim_date, "2024-01-02T00:00:00");
}

#[test]
fn deserializes_balance_object_without_vesting_policy() {
    let object: balance::Object = serde_json::from_value(json!({
        "id": "1.15.4",
        "owner": "BTS1111111111111111111111111111111114T1Anm",
        "balance": {
            "amount": 5_i64,
            "asset_id": "1.3.7"
        },
        "vesting_policy": null,
        "last_claim_date": "1970-01-01T00:00:00"
    }))
    .expect("balance object should deserialize without vesting policy");

    assert_eq!(object.id.to_string(), "1.15.4");
    assert_eq!(object.balance.amount, 5);
    assert_eq!(object.balance.asset_id.to_string(), "1.3.7");
    assert!(object.vesting_policy.is_none());
}

#[test]
fn deserializes_blinded_balance_object() {
    let commitment = "028f7d2c1b00000000000000000000000000000000000000000000000000000000";
    let key = "BTS1111111111111111111111111111111114T1Anm";
    let address = "BTSFN9r6VYzBK8EKtMewfNbfiGCr56pHDBFi";
    let object: blinded_balance::Object = serde_json::from_value(json!({
        "id": "2.10.4",
        "commitment": commitment,
        "asset_id": "1.3.0",
        "owner": {
            "weight_threshold": 2_u32,
            "account_auths": [["1.2.17", 1_u16]],
            "key_auths": [[key, 1_u16]],
            "address_auths": [[address, 1_u16]]
        }
    }))
    .expect("blinded_balance object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.10.4");
    assert_eq!(object.commitment, commitment);
    assert_eq!(object.asset_id.to_string(), "1.3.0");
    assert_eq!(object.owner.weight_threshold, 2);
    assert_eq!(object.owner.account_auths[0].0.to_string(), "1.2.17");
    assert_eq!(object.owner.account_auths[0].1, 1);
    assert_eq!(object.owner.key_auths, vec![(key.to_owned(), 1)]);
    assert_eq!(object.owner.address_auths, vec![(address.to_owned(), 1)]);
}

#[test]
fn deserializes_block_summary_object() {
    let block_id = "0000000a4f3d2c1b000000000000000000000000";
    let object: block_summary::Object = serde_json::from_value(json!({
        "id": "2.8.10",
        "block_id": block_id
    }))
    .expect("block_summary object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.8.10");
    assert_eq!(object.block_id, block_id);
}

#[test]
fn deserializes_buyback_object() {
    let object: buyback::Object = serde_json::from_value(json!({
        "id": "2.15.4",
        "asset_to_buy": "1.3.7"
    }))
    .expect("buyback object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.15.4");
    assert_eq!(object.asset_to_buy.to_string(), "1.3.7");
}

#[test]
fn deserializes_call_order_object_with_target_collateral_ratio() {
    let object: call_order::Object = serde_json::from_value(json!({
        "id": "1.8.9",
        "borrower": "1.2.17",
        "collateral": 10000_i64,
        "debt": 2500_i64,
        "call_price": {
            "base": { "amount": 4_i64, "asset_id": "1.3.7" },
            "quote": { "amount": 1_i64, "asset_id": "1.3.0" }
        },
        "target_collateral_ratio": 1750_u16
    }))
    .expect("call_order object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.8.9");
    assert_eq!(object.borrower.to_string(), "1.2.17");
    assert_eq!(object.collateral, 10000);
    assert_eq!(object.debt, 2500);
    assert_eq!(object.call_price.base.amount, 4);
    assert_eq!(object.call_price.base.asset_id.to_string(), "1.3.7");
    assert_eq!(object.call_price.quote.amount, 1);
    assert_eq!(object.call_price.quote.asset_id.to_string(), "1.3.0");
    assert_eq!(object.target_collateral_ratio, Some(1750));
}

#[test]
fn deserializes_call_order_object_without_target_collateral_ratio() {
    let object: call_order::Object = serde_json::from_value(json!({
        "id": "1.8.10",
        "borrower": "1.2.18",
        "collateral": 100_i64,
        "debt": 20_i64,
        "call_price": {
            "base": { "amount": 5_i64, "asset_id": "1.3.7" },
            "quote": { "amount": 1_i64, "asset_id": "1.3.0" }
        },
        "target_collateral_ratio": null
    }))
    .expect("call_order object should deserialize without target collateral ratio");

    assert_eq!(object.id.to_string(), "1.8.10");
    assert_eq!(object.target_collateral_ratio, None);
}

#[test]
fn deserializes_chain_property_object() {
    let chain_id = "4018d7844c78f6a9f816ed8e2bde14b0df7c6a7ac8f11b6f3b5d6f5e9c8a7b6c";
    let object: chain_property::Object = serde_json::from_value(json!({
        "id": "2.11.0",
        "chain_id": chain_id,
        "immutable_parameters": {
            "min_committee_member_count": 7_u16,
            "min_witness_count": 11_u16,
            "num_special_accounts": 100_u32,
            "num_special_assets": 200_u32
        }
    }))
    .expect("chain_property object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.11.0");
    assert_eq!(object.chain_id, chain_id);
    assert_eq!(object.immutable_parameters.min_committee_member_count, 7);
    assert_eq!(object.immutable_parameters.min_witness_count, 11);
    assert_eq!(object.immutable_parameters.num_special_accounts, 100);
    assert_eq!(object.immutable_parameters.num_special_assets, 200);
}

#[test]
fn deserializes_committee_member_object() {
    let object: committee_member::Object = serde_json::from_value(json!({
        "id": "1.5.2",
        "committee_member_account": "1.2.17",
        "vote_id": "0:12",
        "total_votes": 88_u64,
        "url": "https://committee.example"
    }))
    .expect("committee_member object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.5.2");
    assert_eq!(object.committee_member_account.to_string(), "1.2.17");
    assert_eq!(object.vote_id, "0:12");
    assert_eq!(object.total_votes, 88);
    assert_eq!(object.url, "https://committee.example");
}

#[test]
fn deserializes_collateral_bid_object() {
    let object: collateral_bid::Object = serde_json::from_value(json!({
        "id": "2.17.5",
        "bidder": "1.2.17",
        "inv_swan_price": {
            "base": { "amount": 3_i64, "asset_id": "1.3.7" },
            "quote": { "amount": 2_i64, "asset_id": "1.3.0" }
        }
    }))
    .expect("collateral_bid object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.17.5");
    assert_eq!(object.bidder.to_string(), "1.2.17");
    assert_eq!(object.inv_swan_price.base.amount, 3);
    assert_eq!(object.inv_swan_price.base.asset_id.to_string(), "1.3.7");
    assert_eq!(object.inv_swan_price.quote.amount, 2);
    assert_eq!(object.inv_swan_price.quote.asset_id.to_string(), "1.3.0");
}

#[test]
fn deserializes_credit_deal_object() {
    let object: credit_deal::Object = serde_json::from_value(json!({
        "id": "1.22.8",
        "borrower": "1.2.17",
        "offer_id": "1.21.4",
        "offer_owner": "1.2.18",
        "debt_asset": "1.3.7",
        "debt_amount": 9000_i64,
        "collateral_asset": "1.3.0",
        "collateral_amount": 18000_i64,
        "fee_rate": 250_u32,
        "latest_repay_time": "2024-05-01T00:00:00",
        "auto_repay": 1_u8
    }))
    .expect("credit_deal object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.22.8");
    assert_eq!(object.borrower.to_string(), "1.2.17");
    assert_eq!(object.offer_id.to_string(), "1.21.4");
    assert_eq!(object.offer_owner.to_string(), "1.2.18");
    assert_eq!(object.debt_asset.to_string(), "1.3.7");
    assert_eq!(object.debt_amount, 9000);
    assert_eq!(object.collateral_asset.to_string(), "1.3.0");
    assert_eq!(object.collateral_amount, 18000);
    assert_eq!(object.fee_rate, 250);
    assert_eq!(object.latest_repay_time, "2024-05-01T00:00:00");
    assert_eq!(object.auto_repay, 1);
}

#[test]
fn deserializes_credit_deal_summary_object() {
    let object: credit_deal_summary::Object = serde_json::from_value(json!({
        "id": "2.18.3",
        "borrower": "1.2.17",
        "offer_id": "1.21.4",
        "offer_owner": "1.2.18",
        "debt_asset": "1.3.7",
        "total_debt_amount": 9000_i64
    }))
    .expect("credit_deal_summary object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.18.3");
    assert_eq!(object.borrower.to_string(), "1.2.17");
    assert_eq!(object.offer_id.to_string(), "1.21.4");
    assert_eq!(object.offer_owner.to_string(), "1.2.18");
    assert_eq!(object.debt_asset.to_string(), "1.3.7");
    assert_eq!(object.total_debt_amount, 9000);
}

#[test]
fn deserializes_credit_offer_object() {
    let object: credit_offer::Object = serde_json::from_value(json!({
        "id": "1.21.4",
        "owner_account": "1.2.18",
        "asset_type": "1.3.7",
        "total_balance": 50000_i64,
        "current_balance": 30000_i64,
        "fee_rate": 250_u32,
        "max_duration_seconds": 86400_u32,
        "min_deal_amount": 1000_i64,
        "enabled": true,
        "auto_disable_time": "2024-06-01T00:00:00",
        "acceptable_collateral": [[
            "1.3.0",
            {
                "base": { "amount": 2_i64, "asset_id": "1.3.0" },
                "quote": { "amount": 1_i64, "asset_id": "1.3.7" }
            }
        ]],
        "acceptable_borrowers": [["1.2.17", 10000_i64]]
    }))
    .expect("credit_offer object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.21.4");
    assert_eq!(object.owner_account.to_string(), "1.2.18");
    assert_eq!(object.asset_type.to_string(), "1.3.7");
    assert_eq!(object.total_balance, 50000);
    assert_eq!(object.current_balance, 30000);
    assert_eq!(object.fee_rate, 250);
    assert_eq!(object.max_duration_seconds, 86400);
    assert_eq!(object.min_deal_amount, 1000);
    assert!(object.enabled);
    assert_eq!(object.auto_disable_time, "2024-06-01T00:00:00");
    assert_eq!(object.acceptable_collateral.len(), 1);
    assert_eq!(object.acceptable_collateral[0].0.to_string(), "1.3.0");
    assert_eq!(object.acceptable_collateral[0].1.base.amount, 2);
    assert_eq!(
        object.acceptable_collateral[0].1.base.asset_id.to_string(),
        "1.3.0"
    );
    assert_eq!(object.acceptable_collateral[0].1.quote.amount, 1);
    assert_eq!(
        object.acceptable_collateral[0].1.quote.asset_id.to_string(),
        "1.3.7"
    );
    assert_eq!(
        object.acceptable_borrowers,
        vec![("1.2.17".parse().unwrap(), 10000)]
    );
}

#[test]
fn deserializes_custom_authority_object() {
    let key = "BTS1111111111111111111111111111111114T1Anm";
    let object: custom_authority::Object = serde_json::from_value(json!({
        "id": "1.17.2",
        "account": "1.2.17",
        "enabled": true,
        "valid_from": "2024-01-01T00:00:00",
        "valid_to": "2024-02-01T00:00:00",
        "operation_type": 0_u64,
        "auth": {
            "weight_threshold": 1_u32,
            "account_auths": [],
            "key_auths": [[key, 1_u16]],
            "address_auths": []
        },
        "restrictions": [[
            0_u16,
            {
                "member_index": 1_u64,
                "restriction_type": 0_u64,
                "argument": [1_u64, true],
                "extensions": []
            }
        ]],
        "restriction_counter": 1_u16
    }))
    .expect("custom_authority object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.17.2");
    assert_eq!(object.account.to_string(), "1.2.17");
    assert!(object.enabled);
    assert_eq!(object.valid_from, "2024-01-01T00:00:00");
    assert_eq!(object.valid_to, "2024-02-01T00:00:00");
    assert_eq!(object.operation_type, 0);
    assert_eq!(object.auth.weight_threshold, 1);
    assert_eq!(object.auth.key_auths, vec![(key.to_owned(), 1)]);
    assert_eq!(object.restrictions.len(), 1);
    assert_eq!(object.restrictions[0].0, 0);
    assert_eq!(object.restrictions[0].1.member_index, 1);
    assert_eq!(object.restrictions[0].1.restriction_type, 0);
    assert_eq!(object.restriction_counter, 1);
}

#[test]
fn deserializes_dynamic_global_property_object() {
    let recent_slots_filled = 123456789_u128;
    let object: dynamic_global_property::Object = serde_json::from_value(json!({
        "id": "2.1.0",
        "head_block_number": 123_u32,
        "head_block_id": "0000007b4f3d2c1b000000000000000000000000",
        "time": "2024-01-02T03:04:05",
        "current_witness": "1.6.5",
        "next_maintenance_time": "2024-01-02T04:00:00",
        "last_vote_tally_time": "2024-01-01T00:00:00",
        "last_budget_time": "2024-01-02T02:00:00",
        "witness_budget": -10_i64,
        "total_pob": 20_i64,
        "total_inactive": 30_i64,
        "accounts_registered_this_interval": 4_u32,
        "recently_missed_count": 5_u32,
        "current_aslot": 6_u64,
        "recent_slots_filled": recent_slots_filled,
        "dynamic_flags": 1_u32,
        "last_irreversible_block_num": 122_u32
    }))
    .expect("dynamic_global_property object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.1.0");
    assert_eq!(object.head_block_number, 123);
    assert_eq!(
        object.head_block_id,
        "0000007b4f3d2c1b000000000000000000000000"
    );
    assert_eq!(object.time, "2024-01-02T03:04:05");
    assert_eq!(object.current_witness.to_string(), "1.6.5");
    assert_eq!(object.next_maintenance_time, "2024-01-02T04:00:00");
    assert_eq!(object.last_vote_tally_time, "2024-01-01T00:00:00");
    assert_eq!(object.last_budget_time, "2024-01-02T02:00:00");
    assert_eq!(object.witness_budget, -10);
    assert_eq!(object.total_pob, 20);
    assert_eq!(object.total_inactive, 30);
    assert_eq!(object.accounts_registered_this_interval, 4);
    assert_eq!(object.recently_missed_count, 5);
    assert_eq!(object.current_aslot, 6);
    assert_eq!(object.recent_slots_filled, recent_slots_filled);
    assert_eq!(object.dynamic_flags, 1);
    assert_eq!(object.last_irreversible_block_num, 122);
}

#[test]
fn deserializes_fba_accumulator_object_with_designated_asset() {
    let object: fba_accumulator::Object = serde_json::from_value(json!({
        "id": "2.16.3",
        "accumulated_fba_fees": 999_i64,
        "designated_asset": "1.3.7"
    }))
    .expect("fba_accumulator object should deserialize with designated asset");

    assert_eq!(object.id.to_string(), "2.16.3");
    assert_eq!(object.accumulated_fba_fees, 999);
    assert_eq!(
        object
            .designated_asset
            .expect("asset id should be present")
            .to_string(),
        "1.3.7"
    );
}

#[test]
fn deserializes_fba_accumulator_object_without_designated_asset() {
    let object: fba_accumulator::Object = serde_json::from_value(json!({
        "id": "2.16.4",
        "accumulated_fba_fees": -5_i64,
        "designated_asset": null
    }))
    .expect("fba_accumulator object should deserialize without designated asset");

    assert_eq!(object.id.to_string(), "2.16.4");
    assert_eq!(object.accumulated_fba_fees, -5);
    assert!(object.designated_asset.is_none());
}

#[test]
fn deserializes_force_settlement_object() {
    let object: force_settlement::Object = serde_json::from_value(json!({
        "id": "1.4.3",
        "owner": "1.2.17",
        "balance": {
            "amount": 5000_i64,
            "asset_id": "1.3.7"
        },
        "settlement_date": "2024-01-03T05:06:07"
    }))
    .expect("force_settlement object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.4.3");
    assert_eq!(object.owner.to_string(), "1.2.17");
    assert_eq!(object.balance.amount, 5000);
    assert_eq!(object.balance.asset_id.to_string(), "1.3.7");
    assert_eq!(object.settlement_date, "2024-01-03T05:06:07");
}

#[test]
fn deserializes_worker_object_with_refund_worker() {
    let object: worker::Object = serde_json::from_value(json!({
        "id": "1.14.3",
        "worker_account": "1.2.17",
        "work_begin_date": "2024-01-01T00:00:00",
        "work_end_date": "2024-12-31T23:59:59",
        "daily_pay": 1000_i64,
        "worker": [0_u64, { "total_burned": 25_i64 }],
        "vote_for": "2:10",
        "vote_against": "2:11",
        "total_votes_for": 100_u64,
        "total_votes_against": 5_u64,
        "name": "refund worker",
        "url": "https://worker.example/refund"
    }))
    .expect("worker object should deserialize with refund worker type");

    assert_eq!(object.id.to_string(), "1.14.3");
    assert_eq!(object.worker_account.to_string(), "1.2.17");
    assert_eq!(object.daily_pay, 1000);
    let graphene_protocol::WorkerType::Refund(worker) = object.worker else {
        panic!("expected refund worker");
    };
    assert_eq!(worker.total_burned, 25);
    assert_eq!(object.vote_for, "2:10");
    assert_eq!(object.vote_against, "2:11");
    assert_eq!(object.total_votes_for, 100);
    assert_eq!(object.total_votes_against, 5);
    assert_eq!(object.name, "refund worker");
}

#[test]
fn deserializes_worker_object_with_vesting_balance_worker() {
    let object: worker::Object = serde_json::from_value(json!({
        "id": "1.14.4",
        "worker_account": "1.2.18",
        "work_begin_date": "2024-01-01T00:00:00",
        "work_end_date": "2024-12-31T23:59:59",
        "daily_pay": 2000_i64,
        "worker": [1_u64, { "balance": "1.13.8" }],
        "vote_for": "2:12",
        "vote_against": "2:13",
        "total_votes_for": 200_u64,
        "total_votes_against": 10_u64,
        "name": "vesting worker",
        "url": "https://worker.example/vesting"
    }))
    .expect("worker object should deserialize with vesting balance worker type");

    assert_eq!(object.id.to_string(), "1.14.4");
    let graphene_protocol::WorkerType::VestingBalance(worker) = object.worker else {
        panic!("expected vesting balance worker");
    };
    assert_eq!(worker.balance.to_string(), "1.13.8");
}

#[test]
fn deserializes_witness_object() {
    let signing_key = "BTS1111111111111111111111111111111114T1Anm";
    let object: witness::Object = serde_json::from_value(json!({
        "id": "1.6.5",
        "witness_account": "1.2.17",
        "last_aslot": 123_u64,
        "signing_key": signing_key,
        "pay_vb": "1.13.8",
        "vote_id": "1:5",
        "total_votes": 456_u64,
        "url": "https://witness.example",
        "total_missed": -1_i64,
        "last_confirmed_block_num": 789_u32
    }))
    .expect("witness object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.6.5");
    assert_eq!(object.witness_account.to_string(), "1.2.17");
    assert_eq!(object.last_aslot, 123);
    assert_eq!(object.signing_key, signing_key);
    assert_eq!(
        object
            .pay_vb
            .expect("pay vesting balance should be present")
            .to_string(),
        "1.13.8"
    );
    assert_eq!(object.vote_id, "1:5");
    assert_eq!(object.total_votes, 456);
    assert_eq!(object.url, "https://witness.example");
    assert_eq!(object.total_missed, -1);
    assert_eq!(object.last_confirmed_block_num, 789);
}

#[test]
fn deserializes_witness_schedule_object() {
    let object: witness_schedule::Object = serde_json::from_value(json!({
        "id": "2.12.0",
        "current_shuffled_witnesses": ["1.6.5", "1.6.6"]
    }))
    .expect("witness_schedule object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.12.0");
    let witness_ids = object
        .current_shuffled_witnesses
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(witness_ids, vec!["1.6.5", "1.6.6"]);
}

fn chain_parameters_json() -> serde_json::Value {
    json!({
        "current_fees": { "parameters": [] },
        "block_interval": 3_u8,
        "maintenance_interval": 3600_u32,
        "maintenance_skip_slots": 3_u8,
        "committee_proposal_review_period": 1209600_u32,
        "maximum_transaction_size": 2048_u32,
        "maximum_block_size": 2000000_u32,
        "maximum_time_until_expiration": 86400_u32,
        "maximum_proposal_lifetime": 2419200_u32,
        "maximum_asset_whitelist_authorities": 10_u8,
        "maximum_asset_feed_publishers": 15_u8,
        "maximum_witness_count": 1001_u16,
        "maximum_committee_count": 1001_u16,
        "maximum_authority_membership": 10_u16,
        "reserve_percent_of_fee": 2000_u16,
        "network_percent_of_fee": 2000_u16,
        "lifetime_referrer_percent_of_fee": 3000_u16,
        "cashback_vesting_period_seconds": 31536000_u32,
        "cashback_vesting_threshold": 100000_i64,
        "count_non_member_votes": true,
        "allow_non_member_whitelists": false,
        "witness_pay_per_block": 1000_i64,
        "worker_budget_per_day": 50000_i64,
        "max_predicate_opcode": 1_u16,
        "fee_liquidation_threshold": 1000000_i64,
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
                "max_custom_authorities_per_account": 10_u32,
                "max_custom_authorities_per_account_op": 2_u32,
                "max_custom_authority_restrictions": 5_u32
            },
            "market_fee_network_percent": 100_u16,
            "maker_fee_discount_percent": 50_u16
        }
    })
}

#[test]
fn deserializes_global_property_object() {
    let object: global_property::Object = serde_json::from_value(json!({
        "id": "2.0.0",
        "parameters": chain_parameters_json(),
        "pending_parameters": null,
        "next_available_vote_id": 42_u32,
        "active_committee_members": ["1.5.1", "1.5.2"],
        "active_witnesses": ["1.6.3", "1.6.4"]
    }))
    .expect("global_property object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.0.0");
    assert_eq!(object.parameters.block_interval, 3);
    assert_eq!(object.parameters.cashback_vesting_threshold, 100000);
    assert_eq!(object.parameters.current_fees.0["parameters"], json!([]));
    assert_eq!(
        object
            .parameters
            .extensions
            .updatable_htlc_options
            .expect("htlc options should be present")
            .max_preimage_size,
        1024
    );
    assert!(object.pending_parameters.is_none());
    assert_eq!(object.next_available_vote_id, 42);
    assert_eq!(
        object
            .active_committee_members
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["1.5.1", "1.5.2"]
    );
    assert_eq!(
        object
            .active_witnesses
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["1.6.3", "1.6.4"]
    );
}

#[test]
fn deserializes_htlc_object_with_memo() {
    let object: htlc::Object = serde_json::from_value(json!({
        "id": "1.16.5",
        "transfer": {
            "from": "1.2.17",
            "to": "1.2.18",
            "amount": 5000_i64,
            "asset_id": "1.3.7"
        },
        "conditions": {
            "hash_lock": {
                "preimage_hash": [2_u64, "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"],
                "preimage_size": 32_u16
            },
            "time_lock": {
                "expiration": "2024-08-01T00:00:00"
            }
        },
        "memo": {
            "from": "BTS1111111111111111111111111111111114T1Anm",
            "to": "BTS2222222222222222222222222222222226mYc3t",
            "nonce": 42_u64,
            "message": "deadbeef"
        }
    }))
    .expect("htlc object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.16.5");
    assert_eq!(object.transfer.from.to_string(), "1.2.17");
    assert_eq!(object.transfer.to.to_string(), "1.2.18");
    assert_eq!(object.transfer.amount, 5000);
    assert_eq!(object.transfer.asset_id.to_string(), "1.3.7");
    assert!(matches!(
        object.conditions.hash_lock.preimage_hash,
        graphene_protocol::HtlcHash::Sha256(_)
    ));
    assert_eq!(object.conditions.hash_lock.preimage_size, 32);
    assert_eq!(
        object.conditions.time_lock.expiration,
        "2024-08-01T00:00:00"
    );
    let memo = object.memo.expect("memo should be present");
    assert_eq!(memo.nonce, 42);
}

#[test]
fn deserializes_htlc_object_without_memo() {
    let object: htlc::Object = serde_json::from_value(json!({
        "id": "1.16.6",
        "transfer": {
            "from": "1.2.17",
            "to": "1.2.18",
            "amount": 5000_i64,
            "asset_id": "1.3.7"
        },
        "conditions": {
            "hash_lock": {
                "preimage_hash": [0_u64, "0123456789abcdef0123456789abcdef01234567"],
                "preimage_size": 20_u16
            },
            "time_lock": {
                "expiration": "2024-08-01T00:00:00"
            }
        },
        "memo": null
    }))
    .expect("htlc object should deserialize without memo");

    assert_eq!(object.id.to_string(), "1.16.6");
    assert!(matches!(
        object.conditions.hash_lock.preimage_hash,
        graphene_protocol::HtlcHash::Ripemd160(_)
    ));
    assert!(object.memo.is_none());
}

#[test]
fn deserializes_limit_order_object_with_auto_action() {
    let object: limit_order::Object = serde_json::from_value(json!({
        "id": "1.7.9",
        "expiration": "2024-07-01T00:00:00",
        "seller": "1.2.17",
        "for_sale": 1000_i64,
        "sell_price": {
            "base": { "amount": 2_i64, "asset_id": "1.3.0" },
            "quote": { "amount": 1_i64, "asset_id": "1.3.7" }
        },
        "filled_amount": 25_u128,
        "deferred_fee": 3_i64,
        "deferred_paid_fee": { "amount": 4_i64, "asset_id": "1.3.0" },
        "is_settled_debt": false,
        "on_fill": [[
            0_u64,
            {
                "fee_asset_id": "1.3.0",
                "spread_percent": 100_u16,
                "size_percent": 5000_u16,
                "expiration_seconds": 3600_u32,
                "repeat": true,
                "extensions": []
            }
        ]],
        "take_profit_order_id": "1.7.10"
    }))
    .expect("limit_order object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.7.9");
    assert_eq!(object.expiration, "2024-07-01T00:00:00");
    assert_eq!(object.seller.to_string(), "1.2.17");
    assert_eq!(object.for_sale, 1000);
    assert_eq!(object.sell_price.base.amount, 2);
    assert_eq!(object.sell_price.base.asset_id.to_string(), "1.3.0");
    assert_eq!(object.sell_price.quote.amount, 1);
    assert_eq!(object.sell_price.quote.asset_id.to_string(), "1.3.7");
    assert_eq!(object.filled_amount, 25);
    assert_eq!(object.deferred_fee, 3);
    assert_eq!(object.deferred_paid_fee.amount, 4);
    assert_eq!(object.deferred_paid_fee.asset_id.to_string(), "1.3.0");
    assert!(!object.is_settled_debt);
    assert_eq!(object.on_fill.len(), 1);
    let graphene_protocol::LimitOrderAutoAction::CreateTakeProfitOrder(action) = &object.on_fill[0];
    assert_eq!(action.fee_asset_id.to_string(), "1.3.0");
    assert_eq!(action.spread_percent, 100);
    assert_eq!(action.size_percent, 5000);
    assert_eq!(action.expiration_seconds, 3600);
    assert!(action.repeat);
    assert!(action.extensions.is_empty());
    assert_eq!(
        object
            .take_profit_order_id
            .expect("take profit order id should be present")
            .to_string(),
        "1.7.10"
    );
}

#[test]
fn deserializes_limit_order_object_without_auto_action() {
    let object: limit_order::Object = serde_json::from_value(json!({
        "id": "1.7.11",
        "expiration": "2024-07-01T00:00:00",
        "seller": "1.2.17",
        "for_sale": 1000_i64,
        "sell_price": {
            "base": { "amount": 2_i64, "asset_id": "1.3.0" },
            "quote": { "amount": 1_i64, "asset_id": "1.3.7" }
        },
        "filled_amount": 0_u128,
        "deferred_fee": 0_i64,
        "deferred_paid_fee": { "amount": 0_i64, "asset_id": "1.3.0" },
        "is_settled_debt": false,
        "on_fill": [],
        "take_profit_order_id": null
    }))
    .expect("limit_order object should deserialize without auto action");

    assert_eq!(object.id.to_string(), "1.7.11");
    assert!(object.on_fill.is_empty());
    assert!(object.take_profit_order_id.is_none());
}

#[test]
fn deserializes_liquidity_pool_object() {
    let virtual_value = 2_000_000_u128;
    let object: liquidity_pool::Object = serde_json::from_value(json!({
        "id": "1.19.2",
        "asset_a": "1.3.0",
        "asset_b": "1.3.7",
        "balance_a": 1000_i64,
        "balance_b": 2000_i64,
        "share_asset": "1.3.9",
        "taker_fee_percent": 30_u16,
        "withdrawal_fee_percent": 15_u16,
        "virtual_value": virtual_value
    }))
    .expect("liquidity_pool object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.19.2");
    assert_eq!(object.asset_a.to_string(), "1.3.0");
    assert_eq!(object.asset_b.to_string(), "1.3.7");
    assert_eq!(object.balance_a, 1000);
    assert_eq!(object.balance_b, 2000);
    assert_eq!(object.share_asset.to_string(), "1.3.9");
    assert_eq!(object.taker_fee_percent, 30);
    assert_eq!(object.withdrawal_fee_percent, 15);
    assert_eq!(object.virtual_value, virtual_value);
}

#[test]
fn deserializes_samet_fund_object() {
    let object: samet_fund::Object = serde_json::from_value(json!({
        "id": "1.20.6",
        "owner_account": "1.2.17",
        "asset_type": "1.3.7",
        "balance": 100000_i64,
        "fee_rate": 250_u32,
        "unpaid_amount": 500_i64
    }))
    .expect("samet_fund object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.20.6");
    assert_eq!(object.owner_account.to_string(), "1.2.17");
    assert_eq!(object.asset_type.to_string(), "1.3.7");
    assert_eq!(object.balance, 100000);
    assert_eq!(object.fee_rate, 250);
    assert_eq!(object.unpaid_amount, 500);
}

#[test]
fn deserializes_special_authority_object() {
    let object: special_authority::Object = serde_json::from_value(json!({
        "id": "2.14.3",
        "account": "1.2.17"
    }))
    .expect("special_authority object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "2.14.3");
    assert_eq!(object.account.to_string(), "1.2.17");
}

#[test]
fn deserializes_ticket_object() {
    let object: ticket::Object = serde_json::from_value(json!({
        "id": "1.18.4",
        "account": "1.2.17",
        "target_type": "lock_360_days",
        "amount": {
            "amount": 10000_i64,
            "asset_id": "1.3.0"
        },
        "current_type": "lock_180_days",
        "status": "charging",
        "value": 20000_i64,
        "next_auto_update_time": "2024-03-01T00:00:00",
        "next_type_downgrade_time": "2024-04-01T00:00:00"
    }))
    .expect("ticket object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.18.4");
    assert_eq!(object.account.to_string(), "1.2.17");
    assert_eq!(object.target_type, "lock_360_days");
    assert_eq!(object.amount.amount, 10000);
    assert_eq!(object.amount.asset_id.to_string(), "1.3.0");
    assert_eq!(object.current_type, "lock_180_days");
    assert_eq!(object.status, "charging");
    assert_eq!(object.value, 20000);
    assert_eq!(object.next_auto_update_time, "2024-03-01T00:00:00");
    assert_eq!(object.next_type_downgrade_time, "2024-04-01T00:00:00");
}

#[test]
fn deserializes_vesting_balance_object_with_cdd_policy() {
    let object: vesting_balance::Object = serde_json::from_value(json!({
        "id": "1.13.8",
        "owner": "1.2.17",
        "balance": {
            "amount": 1000_i64,
            "asset_id": "1.3.0"
        },
        "policy": [
            1_u64,
            {
                "vesting_seconds": 86400_u32,
                "start_claim": "2024-01-01T00:00:00",
                "coin_seconds_earned": 123456_u128,
                "coin_seconds_earned_last_update": "2024-01-02T00:00:00"
            }
        ],
        "balance_type": "cashback"
    }))
    .expect("vesting_balance object should deserialize with cdd policy");

    assert_eq!(object.id.to_string(), "1.13.8");
    assert_eq!(object.owner.to_string(), "1.2.17");
    assert_eq!(object.balance.amount, 1000);
    assert_eq!(object.balance.asset_id.to_string(), "1.3.0");
    let graphene_protocol::VestingPolicy::Cdd(policy) = object.policy else {
        panic!("expected cdd vesting policy");
    };
    assert_eq!(policy.vesting_seconds, 86400);
    assert_eq!(policy.coin_seconds_earned, 123456);
    assert_eq!(object.balance_type, "cashback");
}

#[test]
fn deserializes_vesting_balance_object_with_instant_policy() {
    let object: vesting_balance::Object = serde_json::from_value(json!({
        "id": "1.13.9",
        "owner": "1.2.18",
        "balance": {
            "amount": 5_i64,
            "asset_id": "1.3.7"
        },
        "policy": [2_u64, {}],
        "balance_type": "market_fee_sharing"
    }))
    .expect("vesting_balance object should deserialize with instant policy");

    assert_eq!(object.id.to_string(), "1.13.9");
    assert!(matches!(
        object.policy,
        graphene_protocol::VestingPolicy::Instant(_)
    ));
    assert_eq!(object.balance_type, "market_fee_sharing");
}

#[test]
fn deserializes_withdraw_permission_object() {
    let object: withdraw_permission::Object = serde_json::from_value(json!({
        "id": "1.12.9",
        "withdraw_from_account": "1.2.17",
        "authorized_account": "1.2.18",
        "withdrawal_limit": {
            "amount": 5000_i64,
            "asset_id": "1.3.0"
        },
        "withdrawal_period_sec": 3600_u32,
        "period_start_time": "2024-01-02T03:04:05",
        "expiration": "2024-02-02T03:04:05",
        "claimed_this_period": 250_i64
    }))
    .expect("withdraw_permission object should deserialize from Graphene JSON");

    assert_eq!(object.id.to_string(), "1.12.9");
    assert_eq!(object.withdraw_from_account.to_string(), "1.2.17");
    assert_eq!(object.authorized_account.to_string(), "1.2.18");
    assert_eq!(object.withdrawal_limit.amount, 5000);
    assert_eq!(object.withdrawal_limit.asset_id.to_string(), "1.3.0");
    assert_eq!(object.withdrawal_period_sec, 3600);
    assert_eq!(object.period_start_time, "2024-01-02T03:04:05");
    assert_eq!(object.expiration, "2024-02-02T03:04:05");
    assert_eq!(object.claimed_this_period, 250);
}

#[test]
fn rejects_wrong_object_family_id() {
    let error = serde_json::from_value::<account_balance::Object>(json!({
        "id": "2.3.42",
        "owner": "1.2.7",
        "asset_type": "1.3.0",
        "balance": 123_i64,
        "maintenance_flag": false
    }))
    .expect_err("account_balance id should reject asset_dynamic_data object family");

    assert!(
        error.to_string().contains("not a account balance id"),
        "unexpected error: {error}"
    );
}
