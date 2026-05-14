use graphene_chain_acta::{
    broadcast_signed_transaction_synchronous_typed, build_transfer_operation,
    fetch_required_fee_for_transfer, lookup_exact_account_id, prepare_transaction,
    ACTA_TESTNET_FROM_ACCOUNT, ACTA_TESTNET_HTTP_URL, ACTA_TESTNET_TO_ACCOUNT,
    ACTA_TESTNET_TRANSFER_AMOUNT, ACTA_TESTNET_TRANSFER_ASSET, ACTA_TESTNET_WS_URL,
    GetChainIdParams, GetDynamicGlobalPropertiesParams, Operation, TransferDraft,
    PUBLIC_ACTA_TESTNET_WIF,
};
use graphene_rpc::{GrapheneTimePointSec, GrapheneWebSocketTransport, HttpTransport, RpcClient};
use graphene_signing::{ChainId, WifSigner};

fn live_client() -> RpcClient<HttpTransport> {
    RpcClient::new(HttpTransport::new(ACTA_TESTNET_HTTP_URL))
}

fn live_broadcast_client() -> RpcClient<GrapheneWebSocketTransport> {
    RpcClient::new(GrapheneWebSocketTransport::login_api(
        ACTA_TESTNET_WS_URL,
        "network_broadcast",
    ))
}

#[test]
#[ignore = "requires network access and broadcasts a tiny Acta testnet transfer"]
fn live_signs_and_broadcasts_tiny_transfer_with_wif() {
    let signer = WifSigner::from_wif(PUBLIC_ACTA_TESTNET_WIF)
        .expect("public Acta testnet WIF should parse");

    let client = live_client();
    let from = lookup_exact_account_id(&client, ACTA_TESTNET_FROM_ACCOUNT)
        .expect("from account fixture should exist");
    let to = lookup_exact_account_id(&client, ACTA_TESTNET_TO_ACCOUNT)
        .expect("to account fixture should exist");
    assert_ne!(from, to, "live transfer requires distinct accounts");

    let dynamic = client
        .call(GetDynamicGlobalPropertiesParams)
        .expect("dynamic global properties should decode");
    let chain_id = client.call(GetChainIdParams).expect("chain id should decode");
    let chain_id = ChainId::try_from(chain_id.as_str()).expect("chain id should parse");
    let expiration =
        GrapheneTimePointSec::new(dynamic.time.naive_utc() + chrono::Duration::minutes(5));

    let mut transfer = build_transfer_operation(TransferDraft {
        from,
        to,
        amount: ACTA_TESTNET_TRANSFER_AMOUNT,
        asset_id: ACTA_TESTNET_TRANSFER_ASSET.to_owned(),
    })
    .expect("transfer draft should build");
    transfer.fee = fetch_required_fee_for_transfer(
        &client,
        &transfer,
        ACTA_TESTNET_TRANSFER_ASSET,
    )
    .expect("required fee lookup should succeed");

    let prepared = prepare_transaction(&dynamic, vec![Operation::Transfer(transfer)], expiration)
        .expect("transaction should prepare from dynamic global properties");
    let signed = prepared
        .sign(&chain_id, &signer)
        .expect("transaction should sign");
    let broadcast_client = live_broadcast_client();
    let response = broadcast_signed_transaction_synchronous_typed(&broadcast_client, signed)
        .expect("signed transfer should broadcast synchronously");

    println!(
        "broadcast_transaction_synchronous => id={}, block_num={}, trx_num={}",
        response.id, response.block_num, response.trx_num
    );
    assert!(!response.id.is_empty());
}
