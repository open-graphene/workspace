// AUTO-GENERATED legacy Graphene Rust operation variants; do not edit by hand.
// Source: legacy generated OpenRPC swaplock/spec.json (components.schemas)
//
// Each enum corresponds to an fc::static_variant<...> in the C++ protocol.
// On the wire it is the 2-element JSON array `[index, payload]`.

use serde::de::{Deserialize, Deserializer, Error as _DeError};
use serde::ser::{Serialize, SerializeTuple, Serializer};

/// `graphene::protocol::operation` — 78 alternatives.
///
/// Schema: `components.schemas.operation`.
#[derive(Clone, Debug)]
pub enum Operation {
    /// Wire index `0` — C++ `transfer_operation`
    Transfer(TransferOperation),
    /// Wire index `1` — C++ `limit_order_create_operation`
    LimitOrderCreate(LimitOrderCreateOperation),
    /// Wire index `2` — C++ `limit_order_cancel_operation`
    LimitOrderCancel(LimitOrderCancelOperation),
    /// Wire index `3` — C++ `call_order_update_operation`
    CallOrderUpdate(CallOrderUpdateOperation),
    /// Wire index `4` — C++ `fill_order_operation`
    FillOrder(FillOrderOperation),
    /// Wire index `5` — C++ `account_create_operation`
    AccountCreate(AccountCreateOperation),
    /// Wire index `6` — C++ `account_update_operation`
    AccountUpdate(AccountUpdateOperation),
    /// Wire index `7` — C++ `account_whitelist_operation`
    AccountWhitelist(AccountWhitelistOperation),
    /// Wire index `8` — C++ `account_upgrade_operation`
    AccountUpgrade(AccountUpgradeOperation),
    /// Wire index `9` — C++ `account_transfer_operation`
    AccountTransfer(AccountTransferOperation),
    /// Wire index `10` — C++ `asset_create_operation`
    AssetCreate(AssetCreateOperation),
    /// Wire index `11` — C++ `asset_update_operation`
    AssetUpdate(AssetUpdateOperation),
    /// Wire index `12` — C++ `asset_update_bitasset_operation`
    AssetUpdateBitasset(AssetUpdateBitassetOperation),
    /// Wire index `13` — C++ `asset_update_feed_producers_operation`
    AssetUpdateFeedProducers(AssetUpdateFeedProducersOperation),
    /// Wire index `14` — C++ `asset_issue_operation`
    AssetIssue(AssetIssueOperation),
    /// Wire index `15` — C++ `asset_reserve_operation`
    AssetReserve(AssetReserveOperation),
    /// Wire index `16` — C++ `asset_fund_fee_pool_operation`
    AssetFundFeePool(AssetFundFeePoolOperation),
    /// Wire index `17` — C++ `asset_settle_operation`
    AssetSettle(AssetSettleOperation),
    /// Wire index `18` — C++ `asset_global_settle_operation`
    AssetGlobalSettle(AssetGlobalSettleOperation),
    /// Wire index `19` — C++ `asset_publish_feed_operation`
    AssetPublishFeed(AssetPublishFeedOperation),
    /// Wire index `20` — C++ `witness_create_operation`
    WitnessCreate(WitnessCreateOperation),
    /// Wire index `21` — C++ `witness_update_operation`
    WitnessUpdate(WitnessUpdateOperation),
    /// Wire index `22` — C++ `proposal_create_operation`
    ProposalCreate(ProposalCreateOperation),
    /// Wire index `23` — C++ `proposal_update_operation`
    ProposalUpdate(ProposalUpdateOperation),
    /// Wire index `24` — C++ `proposal_delete_operation`
    ProposalDelete(ProposalDeleteOperation),
    /// Wire index `25` — C++ `withdraw_permission_create_operation`
    WithdrawPermissionCreate(WithdrawPermissionCreateOperation),
    /// Wire index `26` — C++ `withdraw_permission_update_operation`
    WithdrawPermissionUpdate(WithdrawPermissionUpdateOperation),
    /// Wire index `27` — C++ `withdraw_permission_claim_operation`
    WithdrawPermissionClaim(WithdrawPermissionClaimOperation),
    /// Wire index `28` — C++ `withdraw_permission_delete_operation`
    WithdrawPermissionDelete(WithdrawPermissionDeleteOperation),
    /// Wire index `29` — C++ `committee_member_create_operation`
    CommitteeMemberCreate(CommitteeMemberCreateOperation),
    /// Wire index `30` — C++ `committee_member_update_operation`
    CommitteeMemberUpdate(CommitteeMemberUpdateOperation),
    /// Wire index `31` — C++ `committee_member_update_global_parameters_operation`
    CommitteeMemberUpdateGlobalParameters(CommitteeMemberUpdateGlobalParametersOperation),
    /// Wire index `32` — C++ `vesting_balance_create_operation`
    VestingBalanceCreate(VestingBalanceCreateOperation),
    /// Wire index `33` — C++ `vesting_balance_withdraw_operation`
    VestingBalanceWithdraw(VestingBalanceWithdrawOperation),
    /// Wire index `34` — C++ `worker_create_operation`
    WorkerCreate(WorkerCreateOperation),
    /// Wire index `35` — C++ `custom_operation`
    Custom(CustomOperation),
    /// Wire index `36` — C++ `assert_operation`
    Assert(AssertOperation),
    /// Wire index `37` — C++ `balance_claim_operation`
    BalanceClaim(BalanceClaimOperation),
    /// Wire index `38` — C++ `override_transfer_operation`
    OverrideTransfer(OverrideTransferOperation),
    /// Wire index `39` — C++ `transfer_to_blind_operation`
    TransferToBlind(TransferToBlindOperation),
    /// Wire index `40` — C++ `blind_transfer_operation`
    BlindTransfer(BlindTransferOperation),
    /// Wire index `41` — C++ `transfer_from_blind_operation`
    TransferFromBlind(TransferFromBlindOperation),
    /// Wire index `42` — C++ `asset_settle_cancel_operation`
    AssetSettleCancel(AssetSettleCancelOperation),
    /// Wire index `43` — C++ `asset_claim_fees_operation`
    AssetClaimFees(AssetClaimFeesOperation),
    /// Wire index `44` — C++ `fba_distribute_operation`
    FbaDistribute(FbaDistributeOperation),
    /// Wire index `45` — C++ `bid_collateral_operation`
    BidCollateral(BidCollateralOperation),
    /// Wire index `46` — C++ `execute_bid_operation`
    ExecuteBid(ExecuteBidOperation),
    /// Wire index `47` — C++ `asset_claim_pool_operation`
    AssetClaimPool(AssetClaimPoolOperation),
    /// Wire index `48` — C++ `asset_update_issuer_operation`
    AssetUpdateIssuer(AssetUpdateIssuerOperation),
    /// Wire index `49` — C++ `htlc_create_operation`
    HtlcCreate(HtlcCreateOperation),
    /// Wire index `50` — C++ `htlc_redeem_operation`
    HtlcRedeem(HtlcRedeemOperation),
    /// Wire index `51` — C++ `htlc_redeemed_operation`
    HtlcRedeemed(HtlcRedeemedOperation),
    /// Wire index `52` — C++ `htlc_extend_operation`
    HtlcExtend(HtlcExtendOperation),
    /// Wire index `53` — C++ `htlc_refund_operation`
    HtlcRefund(HtlcRefundOperation),
    /// Wire index `54` — C++ `custom_authority_create_operation`
    CustomAuthorityCreate(CustomAuthorityCreateOperation),
    /// Wire index `55` — C++ `custom_authority_update_operation`
    CustomAuthorityUpdate(CustomAuthorityUpdateOperation),
    /// Wire index `56` — C++ `custom_authority_delete_operation`
    CustomAuthorityDelete(CustomAuthorityDeleteOperation),
    /// Wire index `57` — C++ `ticket_create_operation`
    TicketCreate(TicketCreateOperation),
    /// Wire index `58` — C++ `ticket_update_operation`
    TicketUpdate(TicketUpdateOperation),
    /// Wire index `59` — C++ `liquidity_pool_create_operation`
    LiquidityPoolCreate(LiquidityPoolCreateOperation),
    /// Wire index `60` — C++ `liquidity_pool_delete_operation`
    LiquidityPoolDelete(LiquidityPoolDeleteOperation),
    /// Wire index `61` — C++ `liquidity_pool_deposit_operation`
    LiquidityPoolDeposit(LiquidityPoolDepositOperation),
    /// Wire index `62` — C++ `liquidity_pool_withdraw_operation`
    LiquidityPoolWithdraw(LiquidityPoolWithdrawOperation),
    /// Wire index `63` — C++ `liquidity_pool_exchange_operation`
    LiquidityPoolExchange(LiquidityPoolExchangeOperation),
    /// Wire index `64` — C++ `samet_fund_create_operation`
    SametFundCreate(SametFundCreateOperation),
    /// Wire index `65` — C++ `samet_fund_delete_operation`
    SametFundDelete(SametFundDeleteOperation),
    /// Wire index `66` — C++ `samet_fund_update_operation`
    SametFundUpdate(SametFundUpdateOperation),
    /// Wire index `67` — C++ `samet_fund_borrow_operation`
    SametFundBorrow(SametFundBorrowOperation),
    /// Wire index `68` — C++ `samet_fund_repay_operation`
    SametFundRepay(SametFundRepayOperation),
    /// Wire index `69` — C++ `credit_offer_create_operation`
    CreditOfferCreate(CreditOfferCreateOperation),
    /// Wire index `70` — C++ `credit_offer_delete_operation`
    CreditOfferDelete(CreditOfferDeleteOperation),
    /// Wire index `71` — C++ `credit_offer_update_operation`
    CreditOfferUpdate(CreditOfferUpdateOperation),
    /// Wire index `72` — C++ `credit_offer_accept_operation`
    CreditOfferAccept(CreditOfferAcceptOperation),
    /// Wire index `73` — C++ `credit_deal_repay_operation`
    CreditDealRepay(CreditDealRepayOperation),
    /// Wire index `74` — C++ `credit_deal_expired_operation`
    CreditDealExpired(CreditDealExpiredOperation),
    /// Wire index `75` — C++ `liquidity_pool_update_operation`
    LiquidityPoolUpdate(LiquidityPoolUpdateOperation),
    /// Wire index `76` — C++ `credit_deal_update_operation`
    CreditDealUpdate(CreditDealUpdateOperation),
    /// Wire index `77` — C++ `limit_order_update_operation`
    LimitOrderUpdate(LimitOrderUpdateOperation),
}

impl Serialize for Operation {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            Operation::Transfer(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            Operation::LimitOrderCreate(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            Operation::LimitOrderCancel(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
            Operation::CallOrderUpdate(value) => { tuple.serialize_element(&3u32)?; tuple.serialize_element(value)?; }
            Operation::FillOrder(value) => { tuple.serialize_element(&4u32)?; tuple.serialize_element(value)?; }
            Operation::AccountCreate(value) => { tuple.serialize_element(&5u32)?; tuple.serialize_element(value)?; }
            Operation::AccountUpdate(value) => { tuple.serialize_element(&6u32)?; tuple.serialize_element(value)?; }
            Operation::AccountWhitelist(value) => { tuple.serialize_element(&7u32)?; tuple.serialize_element(value)?; }
            Operation::AccountUpgrade(value) => { tuple.serialize_element(&8u32)?; tuple.serialize_element(value)?; }
            Operation::AccountTransfer(value) => { tuple.serialize_element(&9u32)?; tuple.serialize_element(value)?; }
            Operation::AssetCreate(value) => { tuple.serialize_element(&10u32)?; tuple.serialize_element(value)?; }
            Operation::AssetUpdate(value) => { tuple.serialize_element(&11u32)?; tuple.serialize_element(value)?; }
            Operation::AssetUpdateBitasset(value) => { tuple.serialize_element(&12u32)?; tuple.serialize_element(value)?; }
            Operation::AssetUpdateFeedProducers(value) => { tuple.serialize_element(&13u32)?; tuple.serialize_element(value)?; }
            Operation::AssetIssue(value) => { tuple.serialize_element(&14u32)?; tuple.serialize_element(value)?; }
            Operation::AssetReserve(value) => { tuple.serialize_element(&15u32)?; tuple.serialize_element(value)?; }
            Operation::AssetFundFeePool(value) => { tuple.serialize_element(&16u32)?; tuple.serialize_element(value)?; }
            Operation::AssetSettle(value) => { tuple.serialize_element(&17u32)?; tuple.serialize_element(value)?; }
            Operation::AssetGlobalSettle(value) => { tuple.serialize_element(&18u32)?; tuple.serialize_element(value)?; }
            Operation::AssetPublishFeed(value) => { tuple.serialize_element(&19u32)?; tuple.serialize_element(value)?; }
            Operation::WitnessCreate(value) => { tuple.serialize_element(&20u32)?; tuple.serialize_element(value)?; }
            Operation::WitnessUpdate(value) => { tuple.serialize_element(&21u32)?; tuple.serialize_element(value)?; }
            Operation::ProposalCreate(value) => { tuple.serialize_element(&22u32)?; tuple.serialize_element(value)?; }
            Operation::ProposalUpdate(value) => { tuple.serialize_element(&23u32)?; tuple.serialize_element(value)?; }
            Operation::ProposalDelete(value) => { tuple.serialize_element(&24u32)?; tuple.serialize_element(value)?; }
            Operation::WithdrawPermissionCreate(value) => { tuple.serialize_element(&25u32)?; tuple.serialize_element(value)?; }
            Operation::WithdrawPermissionUpdate(value) => { tuple.serialize_element(&26u32)?; tuple.serialize_element(value)?; }
            Operation::WithdrawPermissionClaim(value) => { tuple.serialize_element(&27u32)?; tuple.serialize_element(value)?; }
            Operation::WithdrawPermissionDelete(value) => { tuple.serialize_element(&28u32)?; tuple.serialize_element(value)?; }
            Operation::CommitteeMemberCreate(value) => { tuple.serialize_element(&29u32)?; tuple.serialize_element(value)?; }
            Operation::CommitteeMemberUpdate(value) => { tuple.serialize_element(&30u32)?; tuple.serialize_element(value)?; }
            Operation::CommitteeMemberUpdateGlobalParameters(value) => { tuple.serialize_element(&31u32)?; tuple.serialize_element(value)?; }
            Operation::VestingBalanceCreate(value) => { tuple.serialize_element(&32u32)?; tuple.serialize_element(value)?; }
            Operation::VestingBalanceWithdraw(value) => { tuple.serialize_element(&33u32)?; tuple.serialize_element(value)?; }
            Operation::WorkerCreate(value) => { tuple.serialize_element(&34u32)?; tuple.serialize_element(value)?; }
            Operation::Custom(value) => { tuple.serialize_element(&35u32)?; tuple.serialize_element(value)?; }
            Operation::Assert(value) => { tuple.serialize_element(&36u32)?; tuple.serialize_element(value)?; }
            Operation::BalanceClaim(value) => { tuple.serialize_element(&37u32)?; tuple.serialize_element(value)?; }
            Operation::OverrideTransfer(value) => { tuple.serialize_element(&38u32)?; tuple.serialize_element(value)?; }
            Operation::TransferToBlind(value) => { tuple.serialize_element(&39u32)?; tuple.serialize_element(value)?; }
            Operation::BlindTransfer(value) => { tuple.serialize_element(&40u32)?; tuple.serialize_element(value)?; }
            Operation::TransferFromBlind(value) => { tuple.serialize_element(&41u32)?; tuple.serialize_element(value)?; }
            Operation::AssetSettleCancel(value) => { tuple.serialize_element(&42u32)?; tuple.serialize_element(value)?; }
            Operation::AssetClaimFees(value) => { tuple.serialize_element(&43u32)?; tuple.serialize_element(value)?; }
            Operation::FbaDistribute(value) => { tuple.serialize_element(&44u32)?; tuple.serialize_element(value)?; }
            Operation::BidCollateral(value) => { tuple.serialize_element(&45u32)?; tuple.serialize_element(value)?; }
            Operation::ExecuteBid(value) => { tuple.serialize_element(&46u32)?; tuple.serialize_element(value)?; }
            Operation::AssetClaimPool(value) => { tuple.serialize_element(&47u32)?; tuple.serialize_element(value)?; }
            Operation::AssetUpdateIssuer(value) => { tuple.serialize_element(&48u32)?; tuple.serialize_element(value)?; }
            Operation::HtlcCreate(value) => { tuple.serialize_element(&49u32)?; tuple.serialize_element(value)?; }
            Operation::HtlcRedeem(value) => { tuple.serialize_element(&50u32)?; tuple.serialize_element(value)?; }
            Operation::HtlcRedeemed(value) => { tuple.serialize_element(&51u32)?; tuple.serialize_element(value)?; }
            Operation::HtlcExtend(value) => { tuple.serialize_element(&52u32)?; tuple.serialize_element(value)?; }
            Operation::HtlcRefund(value) => { tuple.serialize_element(&53u32)?; tuple.serialize_element(value)?; }
            Operation::CustomAuthorityCreate(value) => { tuple.serialize_element(&54u32)?; tuple.serialize_element(value)?; }
            Operation::CustomAuthorityUpdate(value) => { tuple.serialize_element(&55u32)?; tuple.serialize_element(value)?; }
            Operation::CustomAuthorityDelete(value) => { tuple.serialize_element(&56u32)?; tuple.serialize_element(value)?; }
            Operation::TicketCreate(value) => { tuple.serialize_element(&57u32)?; tuple.serialize_element(value)?; }
            Operation::TicketUpdate(value) => { tuple.serialize_element(&58u32)?; tuple.serialize_element(value)?; }
            Operation::LiquidityPoolCreate(value) => { tuple.serialize_element(&59u32)?; tuple.serialize_element(value)?; }
            Operation::LiquidityPoolDelete(value) => { tuple.serialize_element(&60u32)?; tuple.serialize_element(value)?; }
            Operation::LiquidityPoolDeposit(value) => { tuple.serialize_element(&61u32)?; tuple.serialize_element(value)?; }
            Operation::LiquidityPoolWithdraw(value) => { tuple.serialize_element(&62u32)?; tuple.serialize_element(value)?; }
            Operation::LiquidityPoolExchange(value) => { tuple.serialize_element(&63u32)?; tuple.serialize_element(value)?; }
            Operation::SametFundCreate(value) => { tuple.serialize_element(&64u32)?; tuple.serialize_element(value)?; }
            Operation::SametFundDelete(value) => { tuple.serialize_element(&65u32)?; tuple.serialize_element(value)?; }
            Operation::SametFundUpdate(value) => { tuple.serialize_element(&66u32)?; tuple.serialize_element(value)?; }
            Operation::SametFundBorrow(value) => { tuple.serialize_element(&67u32)?; tuple.serialize_element(value)?; }
            Operation::SametFundRepay(value) => { tuple.serialize_element(&68u32)?; tuple.serialize_element(value)?; }
            Operation::CreditOfferCreate(value) => { tuple.serialize_element(&69u32)?; tuple.serialize_element(value)?; }
            Operation::CreditOfferDelete(value) => { tuple.serialize_element(&70u32)?; tuple.serialize_element(value)?; }
            Operation::CreditOfferUpdate(value) => { tuple.serialize_element(&71u32)?; tuple.serialize_element(value)?; }
            Operation::CreditOfferAccept(value) => { tuple.serialize_element(&72u32)?; tuple.serialize_element(value)?; }
            Operation::CreditDealRepay(value) => { tuple.serialize_element(&73u32)?; tuple.serialize_element(value)?; }
            Operation::CreditDealExpired(value) => { tuple.serialize_element(&74u32)?; tuple.serialize_element(value)?; }
            Operation::LiquidityPoolUpdate(value) => { tuple.serialize_element(&75u32)?; tuple.serialize_element(value)?; }
            Operation::CreditDealUpdate(value) => { tuple.serialize_element(&76u32)?; tuple.serialize_element(value)?; }
            Operation::LimitOrderUpdate(value) => { tuple.serialize_element(&77u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for Operation {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(Operation::Transfer).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(Operation::LimitOrderCreate).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(Operation::LimitOrderCancel).map_err(__D::Error::custom),
            3 => ::serde_json::from_value(payload).map(Operation::CallOrderUpdate).map_err(__D::Error::custom),
            4 => ::serde_json::from_value(payload).map(Operation::FillOrder).map_err(__D::Error::custom),
            5 => ::serde_json::from_value(payload).map(Operation::AccountCreate).map_err(__D::Error::custom),
            6 => ::serde_json::from_value(payload).map(Operation::AccountUpdate).map_err(__D::Error::custom),
            7 => ::serde_json::from_value(payload).map(Operation::AccountWhitelist).map_err(__D::Error::custom),
            8 => ::serde_json::from_value(payload).map(Operation::AccountUpgrade).map_err(__D::Error::custom),
            9 => ::serde_json::from_value(payload).map(Operation::AccountTransfer).map_err(__D::Error::custom),
            10 => ::serde_json::from_value(payload).map(Operation::AssetCreate).map_err(__D::Error::custom),
            11 => ::serde_json::from_value(payload).map(Operation::AssetUpdate).map_err(__D::Error::custom),
            12 => ::serde_json::from_value(payload).map(Operation::AssetUpdateBitasset).map_err(__D::Error::custom),
            13 => ::serde_json::from_value(payload).map(Operation::AssetUpdateFeedProducers).map_err(__D::Error::custom),
            14 => ::serde_json::from_value(payload).map(Operation::AssetIssue).map_err(__D::Error::custom),
            15 => ::serde_json::from_value(payload).map(Operation::AssetReserve).map_err(__D::Error::custom),
            16 => ::serde_json::from_value(payload).map(Operation::AssetFundFeePool).map_err(__D::Error::custom),
            17 => ::serde_json::from_value(payload).map(Operation::AssetSettle).map_err(__D::Error::custom),
            18 => ::serde_json::from_value(payload).map(Operation::AssetGlobalSettle).map_err(__D::Error::custom),
            19 => ::serde_json::from_value(payload).map(Operation::AssetPublishFeed).map_err(__D::Error::custom),
            20 => ::serde_json::from_value(payload).map(Operation::WitnessCreate).map_err(__D::Error::custom),
            21 => ::serde_json::from_value(payload).map(Operation::WitnessUpdate).map_err(__D::Error::custom),
            22 => ::serde_json::from_value(payload).map(Operation::ProposalCreate).map_err(__D::Error::custom),
            23 => ::serde_json::from_value(payload).map(Operation::ProposalUpdate).map_err(__D::Error::custom),
            24 => ::serde_json::from_value(payload).map(Operation::ProposalDelete).map_err(__D::Error::custom),
            25 => ::serde_json::from_value(payload).map(Operation::WithdrawPermissionCreate).map_err(__D::Error::custom),
            26 => ::serde_json::from_value(payload).map(Operation::WithdrawPermissionUpdate).map_err(__D::Error::custom),
            27 => ::serde_json::from_value(payload).map(Operation::WithdrawPermissionClaim).map_err(__D::Error::custom),
            28 => ::serde_json::from_value(payload).map(Operation::WithdrawPermissionDelete).map_err(__D::Error::custom),
            29 => ::serde_json::from_value(payload).map(Operation::CommitteeMemberCreate).map_err(__D::Error::custom),
            30 => ::serde_json::from_value(payload).map(Operation::CommitteeMemberUpdate).map_err(__D::Error::custom),
            31 => ::serde_json::from_value(payload).map(Operation::CommitteeMemberUpdateGlobalParameters).map_err(__D::Error::custom),
            32 => ::serde_json::from_value(payload).map(Operation::VestingBalanceCreate).map_err(__D::Error::custom),
            33 => ::serde_json::from_value(payload).map(Operation::VestingBalanceWithdraw).map_err(__D::Error::custom),
            34 => ::serde_json::from_value(payload).map(Operation::WorkerCreate).map_err(__D::Error::custom),
            35 => ::serde_json::from_value(payload).map(Operation::Custom).map_err(__D::Error::custom),
            36 => ::serde_json::from_value(payload).map(Operation::Assert).map_err(__D::Error::custom),
            37 => ::serde_json::from_value(payload).map(Operation::BalanceClaim).map_err(__D::Error::custom),
            38 => ::serde_json::from_value(payload).map(Operation::OverrideTransfer).map_err(__D::Error::custom),
            39 => ::serde_json::from_value(payload).map(Operation::TransferToBlind).map_err(__D::Error::custom),
            40 => ::serde_json::from_value(payload).map(Operation::BlindTransfer).map_err(__D::Error::custom),
            41 => ::serde_json::from_value(payload).map(Operation::TransferFromBlind).map_err(__D::Error::custom),
            42 => ::serde_json::from_value(payload).map(Operation::AssetSettleCancel).map_err(__D::Error::custom),
            43 => ::serde_json::from_value(payload).map(Operation::AssetClaimFees).map_err(__D::Error::custom),
            44 => ::serde_json::from_value(payload).map(Operation::FbaDistribute).map_err(__D::Error::custom),
            45 => ::serde_json::from_value(payload).map(Operation::BidCollateral).map_err(__D::Error::custom),
            46 => ::serde_json::from_value(payload).map(Operation::ExecuteBid).map_err(__D::Error::custom),
            47 => ::serde_json::from_value(payload).map(Operation::AssetClaimPool).map_err(__D::Error::custom),
            48 => ::serde_json::from_value(payload).map(Operation::AssetUpdateIssuer).map_err(__D::Error::custom),
            49 => ::serde_json::from_value(payload).map(Operation::HtlcCreate).map_err(__D::Error::custom),
            50 => ::serde_json::from_value(payload).map(Operation::HtlcRedeem).map_err(__D::Error::custom),
            51 => ::serde_json::from_value(payload).map(Operation::HtlcRedeemed).map_err(__D::Error::custom),
            52 => ::serde_json::from_value(payload).map(Operation::HtlcExtend).map_err(__D::Error::custom),
            53 => ::serde_json::from_value(payload).map(Operation::HtlcRefund).map_err(__D::Error::custom),
            54 => ::serde_json::from_value(payload).map(Operation::CustomAuthorityCreate).map_err(__D::Error::custom),
            55 => ::serde_json::from_value(payload).map(Operation::CustomAuthorityUpdate).map_err(__D::Error::custom),
            56 => ::serde_json::from_value(payload).map(Operation::CustomAuthorityDelete).map_err(__D::Error::custom),
            57 => ::serde_json::from_value(payload).map(Operation::TicketCreate).map_err(__D::Error::custom),
            58 => ::serde_json::from_value(payload).map(Operation::TicketUpdate).map_err(__D::Error::custom),
            59 => ::serde_json::from_value(payload).map(Operation::LiquidityPoolCreate).map_err(__D::Error::custom),
            60 => ::serde_json::from_value(payload).map(Operation::LiquidityPoolDelete).map_err(__D::Error::custom),
            61 => ::serde_json::from_value(payload).map(Operation::LiquidityPoolDeposit).map_err(__D::Error::custom),
            62 => ::serde_json::from_value(payload).map(Operation::LiquidityPoolWithdraw).map_err(__D::Error::custom),
            63 => ::serde_json::from_value(payload).map(Operation::LiquidityPoolExchange).map_err(__D::Error::custom),
            64 => ::serde_json::from_value(payload).map(Operation::SametFundCreate).map_err(__D::Error::custom),
            65 => ::serde_json::from_value(payload).map(Operation::SametFundDelete).map_err(__D::Error::custom),
            66 => ::serde_json::from_value(payload).map(Operation::SametFundUpdate).map_err(__D::Error::custom),
            67 => ::serde_json::from_value(payload).map(Operation::SametFundBorrow).map_err(__D::Error::custom),
            68 => ::serde_json::from_value(payload).map(Operation::SametFundRepay).map_err(__D::Error::custom),
            69 => ::serde_json::from_value(payload).map(Operation::CreditOfferCreate).map_err(__D::Error::custom),
            70 => ::serde_json::from_value(payload).map(Operation::CreditOfferDelete).map_err(__D::Error::custom),
            71 => ::serde_json::from_value(payload).map(Operation::CreditOfferUpdate).map_err(__D::Error::custom),
            72 => ::serde_json::from_value(payload).map(Operation::CreditOfferAccept).map_err(__D::Error::custom),
            73 => ::serde_json::from_value(payload).map(Operation::CreditDealRepay).map_err(__D::Error::custom),
            74 => ::serde_json::from_value(payload).map(Operation::CreditDealExpired).map_err(__D::Error::custom),
            75 => ::serde_json::from_value(payload).map(Operation::LiquidityPoolUpdate).map_err(__D::Error::custom),
            76 => ::serde_json::from_value(payload).map(Operation::CreditDealUpdate).map_err(__D::Error::custom),
            77 => ::serde_json::from_value(payload).map(Operation::LimitOrderUpdate).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown Operation variant index: {}", other))),
        }
    }
}

/// `graphene::chain::vesting_policy` — 3 alternatives.
///
/// Schema: `components.schemas.vesting_policy`.
#[derive(Clone, Debug)]
pub enum VestingPolicy {
    /// Wire index `0` — C++ `linear_vesting_policy`
    LinearVesting(LinearVestingPolicy),
    /// Wire index `1` — C++ `cdd_vesting_policy`
    CddVesting(CddVestingPolicy),
    /// Wire index `2` — C++ `instant_vesting_policy`
    InstantVesting(InstantVestingPolicy),
}

impl Serialize for VestingPolicy {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            VestingPolicy::LinearVesting(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            VestingPolicy::CddVesting(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            VestingPolicy::InstantVesting(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for VestingPolicy {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(VestingPolicy::LinearVesting).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(VestingPolicy::CddVesting).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(VestingPolicy::InstantVesting).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown VestingPolicy variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::future_extensions` — 1 alternatives.
///
/// Schema: `components.schemas.future_extensions`.
#[derive(Clone, Debug)]
pub enum FutureExtensions {
    /// Wire index `0` — C++ `void_t`
    VoidT(VoidT),
}

impl Serialize for FutureExtensions {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            FutureExtensions::VoidT(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for FutureExtensions {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(FutureExtensions::VoidT).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown FutureExtensions variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::operation_result` — 6 alternatives.
///
/// Schema: `components.schemas.operation_result`.
#[derive(Clone, Debug)]
pub enum OperationResult {
    /// Wire index `0` — C++ `void_result`
    VoidResult(VoidResult),
    /// Wire index `1` — C++ `object_id_type`
    ObjectIdType(::std::string::String),
    /// Wire index `2` — C++ `asset`
    Asset(Asset),
    /// Wire index `3` — C++ `generic_operation_result`
    GenericOperationResult(GenericOperationResult),
    /// Wire index `4` — C++ `generic_exchange_operation_result`
    GenericExchangeOperationResult(GenericExchangeOperationResult),
    /// Wire index `5` — C++ `extendable_operation_result`
    ExtendableOperationResult(ExtendableOperationResult),
}

impl Serialize for OperationResult {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            OperationResult::VoidResult(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            OperationResult::ObjectIdType(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            OperationResult::Asset(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
            OperationResult::GenericOperationResult(value) => { tuple.serialize_element(&3u32)?; tuple.serialize_element(value)?; }
            OperationResult::GenericExchangeOperationResult(value) => { tuple.serialize_element(&4u32)?; tuple.serialize_element(value)?; }
            OperationResult::ExtendableOperationResult(value) => { tuple.serialize_element(&5u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for OperationResult {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(OperationResult::VoidResult).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(OperationResult::ObjectIdType).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(OperationResult::Asset).map_err(__D::Error::custom),
            3 => ::serde_json::from_value(payload).map(OperationResult::GenericOperationResult).map_err(__D::Error::custom),
            4 => ::serde_json::from_value(payload).map(OperationResult::GenericExchangeOperationResult).map_err(__D::Error::custom),
            5 => ::serde_json::from_value(payload).map(OperationResult::ExtendableOperationResult).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown OperationResult variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::limit_order_auto_action` — 1 alternatives.
///
/// Schema: `components.schemas.limit_order_auto_action`.
#[derive(Clone, Debug)]
pub enum LimitOrderAutoAction {
    /// Wire index `0` — C++ `create_take_profit_order_action`
    CreateTakeProfitOrderAction(CreateTakeProfitOrderAction),
}

impl Serialize for LimitOrderAutoAction {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            LimitOrderAutoAction::CreateTakeProfitOrderAction(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for LimitOrderAutoAction {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(LimitOrderAutoAction::CreateTakeProfitOrderAction).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown LimitOrderAutoAction variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::restriction::argument_type` — 42 alternatives.
///
/// Schema: `components.schemas.restriction__argument_type`.
#[derive(Clone, Debug)]
pub enum RestrictionArgumentType {
    /// Wire index `0` — C++ `void_t`
    VoidT(VoidT),
    /// Wire index `1` — C++ `bool`
    Bool(bool),
    /// Wire index `2` — C++ `int64_t`
    Int64T(::graphene_rpc::GrapheneInt64),
    /// Wire index `3` — C++ `string`
    String(::std::string::String),
    /// Wire index `4` — C++ `time_point_sec`
    TimePointSec(::graphene_rpc::GrapheneTimePointSec),
    /// Wire index `5` — C++ `public_key_type`
    PublicKeyType(::std::string::String),
    /// Wire index `6` — C++ `fc::sha256`
    Sha256(::std::string::String),
    /// Wire index `7` — C++ `account_id_type`
    AccountIdType(::std::string::String),
    /// Wire index `8` — C++ `asset_id_type`
    AssetIdType(::std::string::String),
    /// Wire index `9` — C++ `force_settlement_id_type`
    ForceSettlementIdType(::std::string::String),
    /// Wire index `10` — C++ `committee_member_id_type`
    CommitteeMemberIdType(::std::string::String),
    /// Wire index `11` — C++ `witness_id_type`
    WitnessIdType(::std::string::String),
    /// Wire index `12` — C++ `limit_order_id_type`
    LimitOrderIdType(::std::string::String),
    /// Wire index `13` — C++ `call_order_id_type`
    CallOrderIdType(::std::string::String),
    /// Wire index `14` — C++ `custom_id_type`
    CustomIdType(::std::string::String),
    /// Wire index `15` — C++ `proposal_id_type`
    ProposalIdType(::std::string::String),
    /// Wire index `16` — C++ `withdraw_permission_id_type`
    WithdrawPermissionIdType(::std::string::String),
    /// Wire index `17` — C++ `vesting_balance_id_type`
    VestingBalanceIdType(::std::string::String),
    /// Wire index `18` — C++ `worker_id_type`
    WorkerIdType(::std::string::String),
    /// Wire index `19` — C++ `balance_id_type`
    BalanceIdType(::std::string::String),
    /// Wire index `20` — C++ `flat_set<bool>`
    FlatSetBool(::std::vec::Vec<bool>),
    /// Wire index `21` — C++ `flat_set<int64_t>`
    FlatSetInt64T(::std::vec::Vec<::graphene_rpc::GrapheneInt64>),
    /// Wire index `22` — C++ `flat_set<string>`
    FlatSetString(::std::vec::Vec<::std::string::String>),
    /// Wire index `23` — C++ `flat_set<time_point_sec>`
    FlatSetTimePointSec(::std::vec::Vec<::graphene_rpc::GrapheneTimePointSec>),
    /// Wire index `24` — C++ `flat_set<public_key_type>`
    FlatSetPublicKeyType(::std::vec::Vec<::std::string::String>),
    /// Wire index `25` — C++ `flat_set<fc::sha256>`
    Sha2562(::std::vec::Vec<::std::string::String>),
    /// Wire index `26` — C++ `flat_set<account_id_type>`
    FlatSetAccountIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `27` — C++ `flat_set<asset_id_type>`
    FlatSetAssetIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `28` — C++ `flat_set<force_settlement_id_type>`
    FlatSetForceSettlementIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `29` — C++ `flat_set<committee_member_id_type>`
    FlatSetCommitteeMemberIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `30` — C++ `flat_set<witness_id_type>`
    FlatSetWitnessIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `31` — C++ `flat_set<limit_order_id_type>`
    FlatSetLimitOrderIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `32` — C++ `flat_set<call_order_id_type>`
    FlatSetCallOrderIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `33` — C++ `flat_set<custom_id_type>`
    FlatSetCustomIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `34` — C++ `flat_set<proposal_id_type>`
    FlatSetProposalIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `35` — C++ `flat_set<withdraw_permission_id_type>`
    FlatSetWithdrawPermissionIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `36` — C++ `flat_set<vesting_balance_id_type>`
    FlatSetVestingBalanceIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `37` — C++ `flat_set<worker_id_type>`
    FlatSetWorkerIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `38` — C++ `flat_set<balance_id_type>`
    FlatSetBalanceIdType(::std::vec::Vec<::std::string::String>),
    /// Wire index `39` — C++ `vector<restriction>`
    VectorRestriction(::std::vec::Vec<Restriction>),
    /// Wire index `40` — C++ `vector<vector<restriction>>`
    VectorVectorRestriction(::std::vec::Vec<::std::vec::Vec<Restriction>>),
    /// Wire index `41` — C++ `variant_assert_argument_type`
    VariantAssert(RestrictionVariantAssertArgumentType),
}

impl Serialize for RestrictionArgumentType {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            RestrictionArgumentType::VoidT(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::Bool(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::Int64T(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::String(value) => { tuple.serialize_element(&3u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::TimePointSec(value) => { tuple.serialize_element(&4u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::PublicKeyType(value) => { tuple.serialize_element(&5u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::Sha256(value) => { tuple.serialize_element(&6u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::AccountIdType(value) => { tuple.serialize_element(&7u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::AssetIdType(value) => { tuple.serialize_element(&8u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::ForceSettlementIdType(value) => { tuple.serialize_element(&9u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::CommitteeMemberIdType(value) => { tuple.serialize_element(&10u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::WitnessIdType(value) => { tuple.serialize_element(&11u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::LimitOrderIdType(value) => { tuple.serialize_element(&12u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::CallOrderIdType(value) => { tuple.serialize_element(&13u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::CustomIdType(value) => { tuple.serialize_element(&14u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::ProposalIdType(value) => { tuple.serialize_element(&15u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::WithdrawPermissionIdType(value) => { tuple.serialize_element(&16u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::VestingBalanceIdType(value) => { tuple.serialize_element(&17u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::WorkerIdType(value) => { tuple.serialize_element(&18u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::BalanceIdType(value) => { tuple.serialize_element(&19u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetBool(value) => { tuple.serialize_element(&20u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetInt64T(value) => { tuple.serialize_element(&21u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetString(value) => { tuple.serialize_element(&22u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetTimePointSec(value) => { tuple.serialize_element(&23u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetPublicKeyType(value) => { tuple.serialize_element(&24u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::Sha2562(value) => { tuple.serialize_element(&25u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetAccountIdType(value) => { tuple.serialize_element(&26u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetAssetIdType(value) => { tuple.serialize_element(&27u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetForceSettlementIdType(value) => { tuple.serialize_element(&28u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetCommitteeMemberIdType(value) => { tuple.serialize_element(&29u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetWitnessIdType(value) => { tuple.serialize_element(&30u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetLimitOrderIdType(value) => { tuple.serialize_element(&31u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetCallOrderIdType(value) => { tuple.serialize_element(&32u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetCustomIdType(value) => { tuple.serialize_element(&33u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetProposalIdType(value) => { tuple.serialize_element(&34u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetWithdrawPermissionIdType(value) => { tuple.serialize_element(&35u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetVestingBalanceIdType(value) => { tuple.serialize_element(&36u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetWorkerIdType(value) => { tuple.serialize_element(&37u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::FlatSetBalanceIdType(value) => { tuple.serialize_element(&38u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::VectorRestriction(value) => { tuple.serialize_element(&39u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::VectorVectorRestriction(value) => { tuple.serialize_element(&40u32)?; tuple.serialize_element(value)?; }
            RestrictionArgumentType::VariantAssert(value) => { tuple.serialize_element(&41u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for RestrictionArgumentType {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(RestrictionArgumentType::VoidT).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(RestrictionArgumentType::Bool).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(RestrictionArgumentType::Int64T).map_err(__D::Error::custom),
            3 => ::serde_json::from_value(payload).map(RestrictionArgumentType::String).map_err(__D::Error::custom),
            4 => ::serde_json::from_value(payload).map(RestrictionArgumentType::TimePointSec).map_err(__D::Error::custom),
            5 => ::serde_json::from_value(payload).map(RestrictionArgumentType::PublicKeyType).map_err(__D::Error::custom),
            6 => ::serde_json::from_value(payload).map(RestrictionArgumentType::Sha256).map_err(__D::Error::custom),
            7 => ::serde_json::from_value(payload).map(RestrictionArgumentType::AccountIdType).map_err(__D::Error::custom),
            8 => ::serde_json::from_value(payload).map(RestrictionArgumentType::AssetIdType).map_err(__D::Error::custom),
            9 => ::serde_json::from_value(payload).map(RestrictionArgumentType::ForceSettlementIdType).map_err(__D::Error::custom),
            10 => ::serde_json::from_value(payload).map(RestrictionArgumentType::CommitteeMemberIdType).map_err(__D::Error::custom),
            11 => ::serde_json::from_value(payload).map(RestrictionArgumentType::WitnessIdType).map_err(__D::Error::custom),
            12 => ::serde_json::from_value(payload).map(RestrictionArgumentType::LimitOrderIdType).map_err(__D::Error::custom),
            13 => ::serde_json::from_value(payload).map(RestrictionArgumentType::CallOrderIdType).map_err(__D::Error::custom),
            14 => ::serde_json::from_value(payload).map(RestrictionArgumentType::CustomIdType).map_err(__D::Error::custom),
            15 => ::serde_json::from_value(payload).map(RestrictionArgumentType::ProposalIdType).map_err(__D::Error::custom),
            16 => ::serde_json::from_value(payload).map(RestrictionArgumentType::WithdrawPermissionIdType).map_err(__D::Error::custom),
            17 => ::serde_json::from_value(payload).map(RestrictionArgumentType::VestingBalanceIdType).map_err(__D::Error::custom),
            18 => ::serde_json::from_value(payload).map(RestrictionArgumentType::WorkerIdType).map_err(__D::Error::custom),
            19 => ::serde_json::from_value(payload).map(RestrictionArgumentType::BalanceIdType).map_err(__D::Error::custom),
            20 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetBool).map_err(__D::Error::custom),
            21 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetInt64T).map_err(__D::Error::custom),
            22 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetString).map_err(__D::Error::custom),
            23 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetTimePointSec).map_err(__D::Error::custom),
            24 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetPublicKeyType).map_err(__D::Error::custom),
            25 => ::serde_json::from_value(payload).map(RestrictionArgumentType::Sha2562).map_err(__D::Error::custom),
            26 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetAccountIdType).map_err(__D::Error::custom),
            27 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetAssetIdType).map_err(__D::Error::custom),
            28 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetForceSettlementIdType).map_err(__D::Error::custom),
            29 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetCommitteeMemberIdType).map_err(__D::Error::custom),
            30 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetWitnessIdType).map_err(__D::Error::custom),
            31 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetLimitOrderIdType).map_err(__D::Error::custom),
            32 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetCallOrderIdType).map_err(__D::Error::custom),
            33 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetCustomIdType).map_err(__D::Error::custom),
            34 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetProposalIdType).map_err(__D::Error::custom),
            35 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetWithdrawPermissionIdType).map_err(__D::Error::custom),
            36 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetVestingBalanceIdType).map_err(__D::Error::custom),
            37 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetWorkerIdType).map_err(__D::Error::custom),
            38 => ::serde_json::from_value(payload).map(RestrictionArgumentType::FlatSetBalanceIdType).map_err(__D::Error::custom),
            39 => ::serde_json::from_value(payload).map(RestrictionArgumentType::VectorRestriction).map_err(__D::Error::custom),
            40 => ::serde_json::from_value(payload).map(RestrictionArgumentType::VectorVectorRestriction).map_err(__D::Error::custom),
            41 => ::serde_json::from_value(payload).map(RestrictionArgumentType::VariantAssert).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown RestrictionArgumentType variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::htlc_hash` — 4 alternatives.
///
/// Schema: `components.schemas.htlc_hash`.
#[derive(Clone, Debug)]
pub enum HtlcHash {
    /// Wire index `0` — C++ `htlc_algo_ripemd160`
    HtlcAlgoRipemd160(HtlcAlgoRipemd160),
    /// Wire index `1` — C++ `htlc_algo_sha1`
    HtlcAlgoSha1(HtlcAlgoSha1),
    /// Wire index `2` — C++ `htlc_algo_sha256`
    HtlcAlgoSha256(HtlcAlgoSha256),
    /// Wire index `3` — C++ `htlc_algo_hash160`
    HtlcAlgoHash160(HtlcAlgoHash160),
}

impl Serialize for HtlcHash {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            HtlcHash::HtlcAlgoRipemd160(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            HtlcHash::HtlcAlgoSha1(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            HtlcHash::HtlcAlgoSha256(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
            HtlcHash::HtlcAlgoHash160(value) => { tuple.serialize_element(&3u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for HtlcHash {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(HtlcHash::HtlcAlgoRipemd160).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(HtlcHash::HtlcAlgoSha1).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(HtlcHash::HtlcAlgoSha256).map_err(__D::Error::custom),
            3 => ::serde_json::from_value(payload).map(HtlcHash::HtlcAlgoHash160).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown HtlcHash variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::predicate` — 3 alternatives.
///
/// Schema: `components.schemas.predicate`.
#[derive(Clone, Debug)]
pub enum Predicate {
    /// Wire index `0` — C++ `account_name_eq_lit_predicate`
    AccountNameEqLitPredicate(AccountNameEqLitPredicate),
    /// Wire index `1` — C++ `asset_symbol_eq_lit_predicate`
    AssetSymbolEqLitPredicate(AssetSymbolEqLitPredicate),
    /// Wire index `2` — C++ `block_id_predicate`
    BlockIdPredicate(BlockIdPredicate),
}

impl Serialize for Predicate {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            Predicate::AccountNameEqLitPredicate(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            Predicate::AssetSymbolEqLitPredicate(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            Predicate::BlockIdPredicate(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for Predicate {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(Predicate::AccountNameEqLitPredicate).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(Predicate::AssetSymbolEqLitPredicate).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(Predicate::BlockIdPredicate).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown Predicate variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::worker_initializer` — 3 alternatives.
///
/// Schema: `components.schemas.worker_initializer`.
#[derive(Clone, Debug)]
pub enum WorkerInitializer {
    /// Wire index `0` — C++ `refund_worker_initializer`
    RefundWorker(RefundWorkerInitializer),
    /// Wire index `1` — C++ `vesting_balance_worker_initializer`
    VestingBalanceWorker(VestingBalanceWorkerInitializer),
    /// Wire index `2` — C++ `burn_worker_initializer`
    BurnWorker(BurnWorkerInitializer),
}

impl Serialize for WorkerInitializer {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            WorkerInitializer::RefundWorker(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            WorkerInitializer::VestingBalanceWorker(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            WorkerInitializer::BurnWorker(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for WorkerInitializer {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(WorkerInitializer::RefundWorker).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(WorkerInitializer::VestingBalanceWorker).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(WorkerInitializer::BurnWorker).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown WorkerInitializer variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::vesting_policy_initializer` — 3 alternatives.
///
/// Schema: `components.schemas.vesting_policy_initializer`.
#[derive(Clone, Debug)]
pub enum VestingPolicyInitializer {
    /// Wire index `0` — C++ `linear_vesting_policy_initializer`
    LinearVestingPolicy(LinearVestingPolicyInitializer),
    /// Wire index `1` — C++ `cdd_vesting_policy_initializer`
    CddVestingPolicy(CddVestingPolicyInitializer),
    /// Wire index `2` — C++ `instant_vesting_policy_initializer`
    InstantVestingPolicy(InstantVestingPolicyInitializer),
}

impl Serialize for VestingPolicyInitializer {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            VestingPolicyInitializer::LinearVestingPolicy(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            VestingPolicyInitializer::CddVestingPolicy(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            VestingPolicyInitializer::InstantVestingPolicy(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for VestingPolicyInitializer {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(VestingPolicyInitializer::LinearVestingPolicy).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(VestingPolicyInitializer::CddVestingPolicy).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(VestingPolicyInitializer::InstantVestingPolicy).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown VestingPolicyInitializer variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::fee_parameters` — 78 alternatives.
///
/// Schema: `components.schemas.fee_parameters`.
#[derive(Clone, Debug)]
pub enum FeeParameters {
    /// Wire index `0` — C++ `transfer_operation::fee_params_t`
    FeeParamsT(TransferOperationFeeParamsT),
    /// Wire index `1` — C++ `limit_order_create_operation::fee_params_t`
    FeeParamsT2(LimitOrderCreateOperationFeeParamsT),
    /// Wire index `2` — C++ `limit_order_cancel_operation::fee_params_t`
    FeeParamsT3(LimitOrderCancelOperationFeeParamsT),
    /// Wire index `3` — C++ `call_order_update_operation::fee_params_t`
    FeeParamsT4(CallOrderUpdateOperationFeeParamsT),
    /// Wire index `4` — C++ `fill_order_operation::fee_params_t`
    FeeParamsT5(FillOrderOperationFeeParamsT),
    /// Wire index `5` — C++ `account_create_operation::fee_params_t`
    FeeParamsT6(AccountCreateOperationFeeParamsT),
    /// Wire index `6` — C++ `account_update_operation::fee_params_t`
    FeeParamsT7(AccountUpdateOperationFeeParamsT),
    /// Wire index `7` — C++ `account_whitelist_operation::fee_params_t`
    FeeParamsT8(AccountWhitelistOperationFeeParamsT),
    /// Wire index `8` — C++ `account_upgrade_operation::fee_params_t`
    FeeParamsT9(AccountUpgradeOperationFeeParamsT),
    /// Wire index `9` — C++ `account_transfer_operation::fee_params_t`
    FeeParamsT10(AccountTransferOperationFeeParamsT),
    /// Wire index `10` — C++ `asset_create_operation::fee_params_t`
    FeeParamsT11(AssetCreateOperationFeeParamsT),
    /// Wire index `11` — C++ `asset_update_operation::fee_params_t`
    FeeParamsT12(AssetUpdateOperationFeeParamsT),
    /// Wire index `12` — C++ `asset_update_bitasset_operation::fee_params_t`
    FeeParamsT13(AssetUpdateBitassetOperationFeeParamsT),
    /// Wire index `13` — C++ `asset_update_feed_producers_operation::fee_params_t`
    FeeParamsT14(AssetUpdateFeedProducersOperationFeeParamsT),
    /// Wire index `14` — C++ `asset_issue_operation::fee_params_t`
    FeeParamsT15(AssetIssueOperationFeeParamsT),
    /// Wire index `15` — C++ `asset_reserve_operation::fee_params_t`
    FeeParamsT16(AssetReserveOperationFeeParamsT),
    /// Wire index `16` — C++ `asset_fund_fee_pool_operation::fee_params_t`
    FeeParamsT17(AssetFundFeePoolOperationFeeParamsT),
    /// Wire index `17` — C++ `asset_settle_operation::fee_params_t`
    FeeParamsT18(AssetSettleOperationFeeParamsT),
    /// Wire index `18` — C++ `asset_global_settle_operation::fee_params_t`
    FeeParamsT19(AssetGlobalSettleOperationFeeParamsT),
    /// Wire index `19` — C++ `asset_publish_feed_operation::fee_params_t`
    FeeParamsT20(AssetPublishFeedOperationFeeParamsT),
    /// Wire index `20` — C++ `witness_create_operation::fee_params_t`
    FeeParamsT21(WitnessCreateOperationFeeParamsT),
    /// Wire index `21` — C++ `witness_update_operation::fee_params_t`
    FeeParamsT22(WitnessUpdateOperationFeeParamsT),
    /// Wire index `22` — C++ `proposal_create_operation::fee_params_t`
    FeeParamsT23(ProposalCreateOperationFeeParamsT),
    /// Wire index `23` — C++ `proposal_update_operation::fee_params_t`
    FeeParamsT24(ProposalUpdateOperationFeeParamsT),
    /// Wire index `24` — C++ `proposal_delete_operation::fee_params_t`
    FeeParamsT25(ProposalDeleteOperationFeeParamsT),
    /// Wire index `25` — C++ `withdraw_permission_create_operation::fee_params_t`
    FeeParamsT26(WithdrawPermissionCreateOperationFeeParamsT),
    /// Wire index `26` — C++ `withdraw_permission_update_operation::fee_params_t`
    FeeParamsT27(WithdrawPermissionUpdateOperationFeeParamsT),
    /// Wire index `27` — C++ `withdraw_permission_claim_operation::fee_params_t`
    FeeParamsT28(WithdrawPermissionClaimOperationFeeParamsT),
    /// Wire index `28` — C++ `withdraw_permission_delete_operation::fee_params_t`
    FeeParamsT29(WithdrawPermissionDeleteOperationFeeParamsT),
    /// Wire index `29` — C++ `committee_member_create_operation::fee_params_t`
    FeeParamsT30(CommitteeMemberCreateOperationFeeParamsT),
    /// Wire index `30` — C++ `committee_member_update_operation::fee_params_t`
    FeeParamsT31(CommitteeMemberUpdateOperationFeeParamsT),
    /// Wire index `31` — C++ `committee_member_update_global_parameters_operation::fee_params_t`
    FeeParamsT32(CommitteeMemberUpdateGlobalParametersOperationFeeParamsT),
    /// Wire index `32` — C++ `vesting_balance_create_operation::fee_params_t`
    FeeParamsT33(VestingBalanceCreateOperationFeeParamsT),
    /// Wire index `33` — C++ `vesting_balance_withdraw_operation::fee_params_t`
    FeeParamsT34(VestingBalanceWithdrawOperationFeeParamsT),
    /// Wire index `34` — C++ `worker_create_operation::fee_params_t`
    FeeParamsT35(WorkerCreateOperationFeeParamsT),
    /// Wire index `35` — C++ `custom_operation::fee_params_t`
    FeeParamsT36(CustomOperationFeeParamsT),
    /// Wire index `36` — C++ `assert_operation::fee_params_t`
    FeeParamsT37(AssertOperationFeeParamsT),
    /// Wire index `37` — C++ `balance_claim_operation::fee_params_t`
    FeeParamsT38(BalanceClaimOperationFeeParamsT),
    /// Wire index `38` — C++ `override_transfer_operation::fee_params_t`
    FeeParamsT39(OverrideTransferOperationFeeParamsT),
    /// Wire index `39` — C++ `transfer_to_blind_operation::fee_params_t`
    FeeParamsT40(TransferToBlindOperationFeeParamsT),
    /// Wire index `40` — C++ `blind_transfer_operation::fee_params_t`
    FeeParamsT41(BlindTransferOperationFeeParamsT),
    /// Wire index `41` — C++ `transfer_from_blind_operation::fee_params_t`
    FeeParamsT42(TransferFromBlindOperationFeeParamsT),
    /// Wire index `42` — C++ `asset_settle_cancel_operation::fee_params_t`
    FeeParamsT43(AssetSettleCancelOperationFeeParamsT),
    /// Wire index `43` — C++ `asset_claim_fees_operation::fee_params_t`
    FeeParamsT44(AssetClaimFeesOperationFeeParamsT),
    /// Wire index `44` — C++ `fba_distribute_operation::fee_params_t`
    FeeParamsT45(FbaDistributeOperationFeeParamsT),
    /// Wire index `45` — C++ `bid_collateral_operation::fee_params_t`
    FeeParamsT46(BidCollateralOperationFeeParamsT),
    /// Wire index `46` — C++ `execute_bid_operation::fee_params_t`
    FeeParamsT47(ExecuteBidOperationFeeParamsT),
    /// Wire index `47` — C++ `asset_claim_pool_operation::fee_params_t`
    FeeParamsT48(AssetClaimPoolOperationFeeParamsT),
    /// Wire index `48` — C++ `asset_update_issuer_operation::fee_params_t`
    FeeParamsT49(AssetUpdateIssuerOperationFeeParamsT),
    /// Wire index `49` — C++ `htlc_create_operation::fee_params_t`
    FeeParamsT50(HtlcCreateOperationFeeParamsT),
    /// Wire index `50` — C++ `htlc_redeem_operation::fee_params_t`
    FeeParamsT51(HtlcRedeemOperationFeeParamsT),
    /// Wire index `51` — C++ `htlc_redeemed_operation::fee_params_t`
    FeeParamsT52(HtlcRedeemedOperationFeeParamsT),
    /// Wire index `52` — C++ `htlc_extend_operation::fee_params_t`
    FeeParamsT53(HtlcExtendOperationFeeParamsT),
    /// Wire index `53` — C++ `htlc_refund_operation::fee_params_t`
    FeeParamsT54(HtlcRefundOperationFeeParamsT),
    /// Wire index `54` — C++ `custom_authority_create_operation::fee_params_t`
    FeeParamsT55(CustomAuthorityCreateOperationFeeParamsT),
    /// Wire index `55` — C++ `custom_authority_update_operation::fee_params_t`
    FeeParamsT56(CustomAuthorityUpdateOperationFeeParamsT),
    /// Wire index `56` — C++ `custom_authority_delete_operation::fee_params_t`
    FeeParamsT57(CustomAuthorityDeleteOperationFeeParamsT),
    /// Wire index `57` — C++ `ticket_create_operation::fee_params_t`
    FeeParamsT58(TicketCreateOperationFeeParamsT),
    /// Wire index `58` — C++ `ticket_update_operation::fee_params_t`
    FeeParamsT59(TicketUpdateOperationFeeParamsT),
    /// Wire index `59` — C++ `liquidity_pool_create_operation::fee_params_t`
    FeeParamsT60(LiquidityPoolCreateOperationFeeParamsT),
    /// Wire index `60` — C++ `liquidity_pool_delete_operation::fee_params_t`
    FeeParamsT61(LiquidityPoolDeleteOperationFeeParamsT),
    /// Wire index `61` — C++ `liquidity_pool_deposit_operation::fee_params_t`
    FeeParamsT62(LiquidityPoolDepositOperationFeeParamsT),
    /// Wire index `62` — C++ `liquidity_pool_withdraw_operation::fee_params_t`
    FeeParamsT63(LiquidityPoolWithdrawOperationFeeParamsT),
    /// Wire index `63` — C++ `liquidity_pool_exchange_operation::fee_params_t`
    FeeParamsT64(LiquidityPoolExchangeOperationFeeParamsT),
    /// Wire index `64` — C++ `samet_fund_create_operation::fee_params_t`
    FeeParamsT65(SametFundCreateOperationFeeParamsT),
    /// Wire index `65` — C++ `samet_fund_delete_operation::fee_params_t`
    FeeParamsT66(SametFundDeleteOperationFeeParamsT),
    /// Wire index `66` — C++ `samet_fund_update_operation::fee_params_t`
    FeeParamsT67(SametFundUpdateOperationFeeParamsT),
    /// Wire index `67` — C++ `samet_fund_borrow_operation::fee_params_t`
    FeeParamsT68(SametFundBorrowOperationFeeParamsT),
    /// Wire index `68` — C++ `samet_fund_repay_operation::fee_params_t`
    FeeParamsT69(SametFundRepayOperationFeeParamsT),
    /// Wire index `69` — C++ `credit_offer_create_operation::fee_params_t`
    FeeParamsT70(CreditOfferCreateOperationFeeParamsT),
    /// Wire index `70` — C++ `credit_offer_delete_operation::fee_params_t`
    FeeParamsT71(CreditOfferDeleteOperationFeeParamsT),
    /// Wire index `71` — C++ `credit_offer_update_operation::fee_params_t`
    FeeParamsT72(CreditOfferUpdateOperationFeeParamsT),
    /// Wire index `72` — C++ `credit_offer_accept_operation::fee_params_t`
    FeeParamsT73(CreditOfferAcceptOperationFeeParamsT),
    /// Wire index `73` — C++ `credit_deal_repay_operation::fee_params_t`
    FeeParamsT74(CreditDealRepayOperationFeeParamsT),
    /// Wire index `74` — C++ `credit_deal_expired_operation::fee_params_t`
    FeeParamsT75(CreditDealExpiredOperationFeeParamsT),
    /// Wire index `75` — C++ `liquidity_pool_update_operation::fee_params_t`
    FeeParamsT76(LiquidityPoolUpdateOperationFeeParamsT),
    /// Wire index `76` — C++ `credit_deal_update_operation::fee_params_t`
    FeeParamsT77(CreditDealUpdateOperationFeeParamsT),
    /// Wire index `77` — C++ `limit_order_update_operation::fee_params_t`
    FeeParamsT78(LimitOrderUpdateOperationFeeParamsT),
}

impl Serialize for FeeParameters {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            FeeParameters::FeeParamsT(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT2(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT3(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT4(value) => { tuple.serialize_element(&3u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT5(value) => { tuple.serialize_element(&4u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT6(value) => { tuple.serialize_element(&5u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT7(value) => { tuple.serialize_element(&6u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT8(value) => { tuple.serialize_element(&7u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT9(value) => { tuple.serialize_element(&8u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT10(value) => { tuple.serialize_element(&9u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT11(value) => { tuple.serialize_element(&10u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT12(value) => { tuple.serialize_element(&11u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT13(value) => { tuple.serialize_element(&12u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT14(value) => { tuple.serialize_element(&13u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT15(value) => { tuple.serialize_element(&14u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT16(value) => { tuple.serialize_element(&15u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT17(value) => { tuple.serialize_element(&16u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT18(value) => { tuple.serialize_element(&17u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT19(value) => { tuple.serialize_element(&18u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT20(value) => { tuple.serialize_element(&19u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT21(value) => { tuple.serialize_element(&20u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT22(value) => { tuple.serialize_element(&21u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT23(value) => { tuple.serialize_element(&22u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT24(value) => { tuple.serialize_element(&23u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT25(value) => { tuple.serialize_element(&24u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT26(value) => { tuple.serialize_element(&25u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT27(value) => { tuple.serialize_element(&26u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT28(value) => { tuple.serialize_element(&27u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT29(value) => { tuple.serialize_element(&28u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT30(value) => { tuple.serialize_element(&29u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT31(value) => { tuple.serialize_element(&30u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT32(value) => { tuple.serialize_element(&31u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT33(value) => { tuple.serialize_element(&32u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT34(value) => { tuple.serialize_element(&33u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT35(value) => { tuple.serialize_element(&34u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT36(value) => { tuple.serialize_element(&35u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT37(value) => { tuple.serialize_element(&36u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT38(value) => { tuple.serialize_element(&37u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT39(value) => { tuple.serialize_element(&38u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT40(value) => { tuple.serialize_element(&39u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT41(value) => { tuple.serialize_element(&40u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT42(value) => { tuple.serialize_element(&41u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT43(value) => { tuple.serialize_element(&42u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT44(value) => { tuple.serialize_element(&43u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT45(value) => { tuple.serialize_element(&44u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT46(value) => { tuple.serialize_element(&45u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT47(value) => { tuple.serialize_element(&46u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT48(value) => { tuple.serialize_element(&47u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT49(value) => { tuple.serialize_element(&48u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT50(value) => { tuple.serialize_element(&49u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT51(value) => { tuple.serialize_element(&50u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT52(value) => { tuple.serialize_element(&51u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT53(value) => { tuple.serialize_element(&52u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT54(value) => { tuple.serialize_element(&53u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT55(value) => { tuple.serialize_element(&54u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT56(value) => { tuple.serialize_element(&55u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT57(value) => { tuple.serialize_element(&56u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT58(value) => { tuple.serialize_element(&57u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT59(value) => { tuple.serialize_element(&58u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT60(value) => { tuple.serialize_element(&59u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT61(value) => { tuple.serialize_element(&60u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT62(value) => { tuple.serialize_element(&61u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT63(value) => { tuple.serialize_element(&62u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT64(value) => { tuple.serialize_element(&63u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT65(value) => { tuple.serialize_element(&64u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT66(value) => { tuple.serialize_element(&65u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT67(value) => { tuple.serialize_element(&66u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT68(value) => { tuple.serialize_element(&67u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT69(value) => { tuple.serialize_element(&68u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT70(value) => { tuple.serialize_element(&69u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT71(value) => { tuple.serialize_element(&70u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT72(value) => { tuple.serialize_element(&71u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT73(value) => { tuple.serialize_element(&72u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT74(value) => { tuple.serialize_element(&73u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT75(value) => { tuple.serialize_element(&74u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT76(value) => { tuple.serialize_element(&75u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT77(value) => { tuple.serialize_element(&76u32)?; tuple.serialize_element(value)?; }
            FeeParameters::FeeParamsT78(value) => { tuple.serialize_element(&77u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for FeeParameters {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT2).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT3).map_err(__D::Error::custom),
            3 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT4).map_err(__D::Error::custom),
            4 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT5).map_err(__D::Error::custom),
            5 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT6).map_err(__D::Error::custom),
            6 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT7).map_err(__D::Error::custom),
            7 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT8).map_err(__D::Error::custom),
            8 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT9).map_err(__D::Error::custom),
            9 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT10).map_err(__D::Error::custom),
            10 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT11).map_err(__D::Error::custom),
            11 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT12).map_err(__D::Error::custom),
            12 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT13).map_err(__D::Error::custom),
            13 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT14).map_err(__D::Error::custom),
            14 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT15).map_err(__D::Error::custom),
            15 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT16).map_err(__D::Error::custom),
            16 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT17).map_err(__D::Error::custom),
            17 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT18).map_err(__D::Error::custom),
            18 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT19).map_err(__D::Error::custom),
            19 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT20).map_err(__D::Error::custom),
            20 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT21).map_err(__D::Error::custom),
            21 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT22).map_err(__D::Error::custom),
            22 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT23).map_err(__D::Error::custom),
            23 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT24).map_err(__D::Error::custom),
            24 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT25).map_err(__D::Error::custom),
            25 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT26).map_err(__D::Error::custom),
            26 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT27).map_err(__D::Error::custom),
            27 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT28).map_err(__D::Error::custom),
            28 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT29).map_err(__D::Error::custom),
            29 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT30).map_err(__D::Error::custom),
            30 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT31).map_err(__D::Error::custom),
            31 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT32).map_err(__D::Error::custom),
            32 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT33).map_err(__D::Error::custom),
            33 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT34).map_err(__D::Error::custom),
            34 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT35).map_err(__D::Error::custom),
            35 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT36).map_err(__D::Error::custom),
            36 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT37).map_err(__D::Error::custom),
            37 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT38).map_err(__D::Error::custom),
            38 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT39).map_err(__D::Error::custom),
            39 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT40).map_err(__D::Error::custom),
            40 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT41).map_err(__D::Error::custom),
            41 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT42).map_err(__D::Error::custom),
            42 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT43).map_err(__D::Error::custom),
            43 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT44).map_err(__D::Error::custom),
            44 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT45).map_err(__D::Error::custom),
            45 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT46).map_err(__D::Error::custom),
            46 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT47).map_err(__D::Error::custom),
            47 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT48).map_err(__D::Error::custom),
            48 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT49).map_err(__D::Error::custom),
            49 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT50).map_err(__D::Error::custom),
            50 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT51).map_err(__D::Error::custom),
            51 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT52).map_err(__D::Error::custom),
            52 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT53).map_err(__D::Error::custom),
            53 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT54).map_err(__D::Error::custom),
            54 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT55).map_err(__D::Error::custom),
            55 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT56).map_err(__D::Error::custom),
            56 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT57).map_err(__D::Error::custom),
            57 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT58).map_err(__D::Error::custom),
            58 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT59).map_err(__D::Error::custom),
            59 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT60).map_err(__D::Error::custom),
            60 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT61).map_err(__D::Error::custom),
            61 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT62).map_err(__D::Error::custom),
            62 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT63).map_err(__D::Error::custom),
            63 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT64).map_err(__D::Error::custom),
            64 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT65).map_err(__D::Error::custom),
            65 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT66).map_err(__D::Error::custom),
            66 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT67).map_err(__D::Error::custom),
            67 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT68).map_err(__D::Error::custom),
            68 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT69).map_err(__D::Error::custom),
            69 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT70).map_err(__D::Error::custom),
            70 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT71).map_err(__D::Error::custom),
            71 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT72).map_err(__D::Error::custom),
            72 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT73).map_err(__D::Error::custom),
            73 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT74).map_err(__D::Error::custom),
            74 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT75).map_err(__D::Error::custom),
            75 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT76).map_err(__D::Error::custom),
            76 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT77).map_err(__D::Error::custom),
            77 => ::serde_json::from_value(payload).map(FeeParameters::FeeParamsT78).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown FeeParameters variant index: {}", other))),
        }
    }
}

/// `graphene::protocol::special_authority` — 2 alternatives.
///
/// Schema: `components.schemas.special_authority`.
#[derive(Clone, Debug)]
pub enum SpecialAuthority {
    /// Wire index `0` — C++ `no_special_authority`
    NoSpecial(NoSpecialAuthority),
    /// Wire index `1` — C++ `top_holders_special_authority`
    TopHoldersSpecial(TopHoldersSpecialAuthority),
}

impl Serialize for SpecialAuthority {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            SpecialAuthority::NoSpecial(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            SpecialAuthority::TopHoldersSpecial(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for SpecialAuthority {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(SpecialAuthority::NoSpecial).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(SpecialAuthority::TopHoldersSpecial).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown SpecialAuthority variant index: {}", other))),
        }
    }
}

/// `graphene::chain::worker_type` — 3 alternatives.
///
/// Schema: `components.schemas.worker_type`.
#[derive(Clone, Debug)]
pub enum WorkerType {
    /// Wire index `0` — C++ `refund_worker_type`
    RefundWorkerType(RefundWorkerType),
    /// Wire index `1` — C++ `vesting_balance_worker_type`
    VestingBalanceWorkerType(VestingBalanceWorkerType),
    /// Wire index `2` — C++ `burn_worker_type`
    BurnWorkerType(BurnWorkerType),
}

impl Serialize for WorkerType {
    fn serialize<__S: Serializer>(&self, ser: __S) -> ::core::result::Result<__S::Ok, __S::Error> {
        let mut tuple = ser.serialize_tuple(2)?;
        match self {
            WorkerType::RefundWorkerType(value) => { tuple.serialize_element(&0u32)?; tuple.serialize_element(value)?; }
            WorkerType::VestingBalanceWorkerType(value) => { tuple.serialize_element(&1u32)?; tuple.serialize_element(value)?; }
            WorkerType::BurnWorkerType(value) => { tuple.serialize_element(&2u32)?; tuple.serialize_element(value)?; }
        }
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for WorkerType {
    fn deserialize<__D: Deserializer<'de>>(de: __D) -> ::core::result::Result<Self, __D::Error> {
        let (index, payload): (u32, ::serde_json::Value) = Deserialize::deserialize(de)?;
        match index {
            0 => ::serde_json::from_value(payload).map(WorkerType::RefundWorkerType).map_err(__D::Error::custom),
            1 => ::serde_json::from_value(payload).map(WorkerType::VestingBalanceWorkerType).map_err(__D::Error::custom),
            2 => ::serde_json::from_value(payload).map(WorkerType::BurnWorkerType).map_err(__D::Error::custom),
            other => Err(__D::Error::custom(format!("unknown WorkerType variant index: {}", other))),
        }
    }
}
