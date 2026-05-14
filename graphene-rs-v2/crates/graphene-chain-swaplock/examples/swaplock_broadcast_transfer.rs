use graphene_chain_swaplock::{
    broadcast_signed_transaction_synchronous_typed, build_transfer_operation,
    fetch_required_fee_for_transfer, prepare_transaction, GetChainIdParams,
    GetDynamicGlobalPropertiesParams, LookupAccountsParams, Operation, TransferDraft,
};
use graphene_rpc::{GrapheneTimePointSec, GrapheneWebSocketTransport, HttpTransport, RpcClient};
use graphene_signing::{ChainId, WifSigner};

const HTTP_URL: &str = "https://node01.swaplock.chainpool.online:8090";
const WS_URL: &str = "wss://node01.swaplock.chainpool.online:8090";
const FROM_ACCOUNT: &str = "swaplock";
const TO_ACCOUNT: &str = "committee-account";

/// Public Swaplock testnet-only WIF. Do not copy this pattern for mainnet keys.
const SWAPLOCK_TESTNET_WIF: &str = "5K71C3PVyynjDdzxNdgd5YJ6y8Z86eEc5RNPvAN983UdhCF4HPw";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_client = RpcClient::new(HttpTransport::new(HTTP_URL));
    let broadcast_client = RpcClient::new(GrapheneWebSocketTransport::login_api(
        WS_URL,
        "network_broadcast",
    ));
    let signer = WifSigner::from_wif(SWAPLOCK_TESTNET_WIF)?;

    let from = exact_account_id(&database_client, FROM_ACCOUNT)?;
    let to = exact_account_id(&database_client, TO_ACCOUNT)?;
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
