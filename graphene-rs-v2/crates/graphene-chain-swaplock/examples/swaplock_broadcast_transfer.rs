use graphene_chain_swaplock::{
    broadcast_signed_transaction_synchronous, build_transfer_operation,
    fetch_required_fee_for_transfer, prepare_transaction, sign_transaction, GetChainIdParams,
    GetDynamicGlobalPropertiesParams, LookupAccountsParams, Operation, TransferDraft,
};
use graphene_rpc::{GrapheneTimePointSec, GrapheneWebSocketTransport, HttpTransport, RpcClient};
use graphene_signing::{ChainId, WifSigner};

const DEFAULT_HTTP_URL: &str = "https://node01.swaplock.chainpool.online:8090";
const DEFAULT_WS_URL: &str = "wss://node01.swaplock.chainpool.online:8090";

/// Public Swaplock testnet-only WIF. Do not copy this pattern for mainnet keys.
const SWAPLOCK_TESTNET_WIF: &str = "5K71C3PVyynjDdzxNdgd5YJ6y8Z86eEc5RNPvAN983UdhCF4HPw";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_url = std::env::var("SWAPLOCK_RPC_URL").unwrap_or_else(|_| DEFAULT_HTTP_URL.into());
    let ws_url = std::env::var("SWAPLOCK_WS_URL").unwrap_or_else(|_| DEFAULT_WS_URL.into());
    let from_account = std::env::var("SWAPLOCK_FROM_ACCOUNT").unwrap_or_else(|_| "swaplock".into());
    let to_account =
        std::env::var("SWAPLOCK_TO_ACCOUNT").unwrap_or_else(|_| "committee-account".into());

    let database_client = RpcClient::new(HttpTransport::new(http_url));
    let broadcast_client = RpcClient::new(GrapheneWebSocketTransport::login_api(
        ws_url,
        "network_broadcast",
    ));
    let signer = WifSigner::from_wif(SWAPLOCK_TESTNET_WIF)?;

    let from = exact_account_id(&database_client, &from_account)?;
    let to = exact_account_id(&database_client, &to_account)?;
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

    let transaction =
        prepare_transaction(&dynamic, vec![Operation::Transfer(transfer)], expiration)?;
    let signed = sign_transaction(&chain_id, transaction, &signer)?;
    let response = broadcast_signed_transaction_synchronous(&broadcast_client, signed)?;

    let tx_id = response
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or("broadcast response is missing transaction id")?;
    let block_num = response
        .get("block_num")
        .and_then(serde_json::Value::as_u64)
        .ok_or("broadcast response is missing block number")?;
    let trx_num = response
        .get("trx_num")
        .and_then(serde_json::Value::as_u64)
        .ok_or("broadcast response is missing transaction number")?;

    println!(
        "broadcast_transaction_synchronous => id={tx_id}, block_num={block_num}, trx_num={trx_num}"
    );
    Ok(())
}

fn exact_account_id(
    client: &RpcClient<HttpTransport>,
    account_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let accounts = client.call(LookupAccountsParams {
        lower_bound_name: account_name.to_owned(),
        limit: 1,
        subscribe: Some(false),
    })?;
    let (name, id) = accounts
        .iter()
        .next()
        .ok_or("account lookup returned no rows")?;
    if name != account_name {
        return Err(format!("expected account {account_name}, got {name}").into());
    }
    Ok(id.clone())
}
