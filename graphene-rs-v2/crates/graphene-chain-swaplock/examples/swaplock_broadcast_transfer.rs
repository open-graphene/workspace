use graphene_chain_swaplock::{
    broadcast_signed_transaction_synchronous_typed, lookup_exact_account_id,
    prepare_transfer_transaction, GetChainIdParams, TransferDraft, PUBLIC_SWAPLOCK_TESTNET_WIF,
    SWAPLOCK_TESTNET_FROM_ACCOUNT, SWAPLOCK_TESTNET_HTTP_URL, SWAPLOCK_TESTNET_TO_ACCOUNT,
    SWAPLOCK_TESTNET_TRANSFER_AMOUNT, SWAPLOCK_TESTNET_TRANSFER_ASSET, SWAPLOCK_TESTNET_WS_URL,
};
use graphene_rpc::{GrapheneWebSocketSession, HttpTransport, RpcClient};
use graphene_signing::{ChainId, WifSigner};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_client = RpcClient::new(HttpTransport::new(SWAPLOCK_TESTNET_HTTP_URL));
    let broadcast_session = Arc::new(GrapheneWebSocketSession::connect(SWAPLOCK_TESTNET_WS_URL)?);
    let broadcast_api = broadcast_session.login_api("network_broadcast")?;
    let broadcast_client = RpcClient::new(broadcast_session.api_transport(&broadcast_api));
    let signer = WifSigner::from_wif(PUBLIC_SWAPLOCK_TESTNET_WIF)?;

    let from = lookup_exact_account_id(&database_client, SWAPLOCK_TESTNET_FROM_ACCOUNT)?;
    let to = lookup_exact_account_id(&database_client, SWAPLOCK_TESTNET_TO_ACCOUNT)?;
    if from == to {
        return Err("from and to accounts must be distinct".into());
    }

    let chain_id = database_client.call(GetChainIdParams)?;
    let chain_id = ChainId::try_from(chain_id.as_str())?;

    let prepared = prepare_transfer_transaction(
        &database_client,
        TransferDraft {
            from,
            to,
            amount: SWAPLOCK_TESTNET_TRANSFER_AMOUNT,
            asset_id: SWAPLOCK_TESTNET_TRANSFER_ASSET.to_owned(),
        },
        SWAPLOCK_TESTNET_TRANSFER_ASSET,
        chrono::Duration::minutes(5),
    )?;
    let signed = prepared.sign(&chain_id, &signer)?;
    let response = broadcast_signed_transaction_synchronous_typed(&broadcast_client, signed)?;

    println!(
        "broadcast_transaction_synchronous => id={}, block_num={}, trx_num={}",
        response.id, response.block_num, response.trx_num
    );
    Ok(())
}
