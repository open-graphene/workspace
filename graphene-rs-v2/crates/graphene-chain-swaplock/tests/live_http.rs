use graphene_chain_swaplock::{GetAccountCountParams, GetDynamicGlobalPropertiesParams};
use graphene_rpc::{HttpTransport, RpcClient};

const DEFAULT_SWAPLOCK_RPC_URL: &str = "https://node01.swaplock.chainpool.online:8090";

#[test]
#[ignore = "requires network access to the Swaplock testnet RPC endpoint"]
fn live_decodes_dynamic_global_properties_over_http() {
    let endpoint =
        std::env::var("SWAPLOCK_RPC_URL").unwrap_or_else(|_| DEFAULT_SWAPLOCK_RPC_URL.to_owned());
    let client = RpcClient::new(HttpTransport::new(endpoint));

    let properties = client.call(GetDynamicGlobalPropertiesParams).unwrap();

    println!("{properties:#?}");
    assert!(properties.head_block_number > 0);
}

#[test]
#[ignore = "requires network access to the Swaplock testnet RPC endpoint"]
fn live_get_account_count_over_http() {
    let endpoint =
        std::env::var("SWAPLOCK_RPC_URL").unwrap_or_else(|_| DEFAULT_SWAPLOCK_RPC_URL.to_owned());
    let client = RpcClient::new(HttpTransport::new(endpoint));

    let account_count = client.call(GetAccountCountParams).unwrap();

    println!("{account_count}");
    assert!(account_count > 0);
}
