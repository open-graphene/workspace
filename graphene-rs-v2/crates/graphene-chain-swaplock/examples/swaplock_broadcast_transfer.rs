use graphene_chain_swaplock::{
    broadcast_signed_transaction_synchronous_typed, build_transfer_operation,
    fetch_required_fee_for_transfer, lookup_exact_account_id, prepare_transaction,
    GetChainIdParams, GetDynamicGlobalPropertiesParams, Operation, TransferDraft,
    PUBLIC_SWAPLOCK_TESTNET_WIF, SWAPLOCK_TESTNET_FROM_ACCOUNT, SWAPLOCK_TESTNET_HTTP_URL,
    SWAPLOCK_TESTNET_TO_ACCOUNT, SWAPLOCK_TESTNET_WS_URL,
};
use graphene_rpc::{GrapheneTimePointSec, GrapheneWebSocketTransport, HttpTransport, RpcClient};
use graphene_signing::{ChainId, WifSigner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_client = RpcClient::new(HttpTransport::new(SWAPLOCK_TESTNET_HTTP_URL));
    let broadcast_client = RpcClient::new(GrapheneWebSocketTransport::login_api(
        SWAPLOCK_TESTNET_WS_URL,
        "network_broadcast",
    ));
    let signer = WifSigner::from_wif(PUBLIC_SWAPLOCK_TESTNET_WIF)?;

    let from = lookup_exact_account_id(&database_client, SWAPLOCK_TESTNET_FROM_ACCOUNT)?;
    let to = lookup_exact_account_id(&database_client, SWAPLOCK_TESTNET_TO_ACCOUNT)?;
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
        amount: 1,
        asset_id: "1.3.0".to_owned(),
    })?;
    transfer.fee = fetch_required_fee_for_transfer(&database_client, &transfer, "1.3.0")?;

    let prepared = prepare_transaction(&dynamic, vec![Operation::Transfer(transfer)], expiration)?;
    let signed = prepared.sign(&chain_id, &signer)?;
    let response = broadcast_signed_transaction_synchronous_typed(&broadcast_client, signed)?;

    println!(
        "broadcast_transaction_synchronous => id={}, block_num={}, trx_num={}",
        response.id, response.block_num, response.trx_num
    );
    Ok(())
}
