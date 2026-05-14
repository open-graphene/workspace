use graphene_chain_acta::{
    broadcast_signed_transaction_synchronous_typed, build_transfer_operation,
    fetch_required_fee_for_transfer, lookup_exact_account_id, prepare_transaction,
    GetChainIdParams, GetDynamicGlobalPropertiesParams, Operation, TransferDraft,
    ACTA_TESTNET_FROM_ACCOUNT, ACTA_TESTNET_HTTP_URL, ACTA_TESTNET_TO_ACCOUNT,
    ACTA_TESTNET_TRANSFER_AMOUNT, ACTA_TESTNET_TRANSFER_ASSET, ACTA_TESTNET_WS_URL,
    PUBLIC_ACTA_TESTNET_WIF,
};
use graphene_rpc::{GrapheneTimePointSec, GrapheneWebSocketSession, HttpTransport, RpcClient};
use graphene_signing::{ChainId, WifSigner};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_client = RpcClient::new(HttpTransport::new(ACTA_TESTNET_HTTP_URL));
    let broadcast_session = Arc::new(GrapheneWebSocketSession::connect(ACTA_TESTNET_WS_URL)?);
    let broadcast_api = broadcast_session.login_api("network_broadcast")?;
    let broadcast_client = RpcClient::new(broadcast_session.api_transport(&broadcast_api));
    let signer = WifSigner::from_wif(PUBLIC_ACTA_TESTNET_WIF)?;

    let from = lookup_exact_account_id(&database_client, ACTA_TESTNET_FROM_ACCOUNT)?;
    let to = lookup_exact_account_id(&database_client, ACTA_TESTNET_TO_ACCOUNT)?;
    if from == to {
        return Err("from and to accounts must be distinct".into());
    }

    let dynamic = database_client.call(GetDynamicGlobalPropertiesParams)?;
    let chain_id = database_client.call(GetChainIdParams)?;
    let chain_id = ChainId::try_from(chain_id.as_str())?;
    let expiration =
        GrapheneTimePointSec::new(dynamic.time.naive_utc() + chrono::Duration::minutes(5));

    let mut transfer = build_transfer_operation(TransferDraft {
        from,
        to,
        amount: ACTA_TESTNET_TRANSFER_AMOUNT,
        asset_id: ACTA_TESTNET_TRANSFER_ASSET.to_owned(),
    })?;
    transfer.fee =
        fetch_required_fee_for_transfer(&database_client, &transfer, ACTA_TESTNET_TRANSFER_ASSET)?;

    let prepared = prepare_transaction(&dynamic, vec![Operation::Transfer(transfer)], expiration)?;
    let signed = prepared.sign(&chain_id, &signer)?;
    let response = broadcast_signed_transaction_synchronous_typed(&broadcast_client, signed)?;

    println!(
        "broadcast_transaction_synchronous => id={}, block_num={}, trx_num={}",
        response.id, response.block_num, response.trx_num
    );
    Ok(())
}
