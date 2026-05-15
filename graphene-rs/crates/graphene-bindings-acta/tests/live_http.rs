use graphene_bindings_acta::{
    broadcast_signed_transaction_synchronous_typed,
    broadcast_signed_transaction_with_callback_typed, lookup_exact_account_id,
    prepare_transfer_transaction, set_block_applied_callback, subscribe_dynamic_global_properties,
    GetChainIdParams, TransferDraft, ACTA_TESTNET_FROM_ACCOUNT, ACTA_TESTNET_HTTP_URL,
    ACTA_TESTNET_TO_ACCOUNT, ACTA_TESTNET_TRANSFER_AMOUNT, ACTA_TESTNET_TRANSFER_ASSET,
    ACTA_TESTNET_WS_URL, PUBLIC_ACTA_TESTNET_WIF,
};
use graphene_rpc::{
    GrapheneWebSocketApiTransport, GrapheneWebSocketSession, HttpTransport, RpcClient,
};
use graphene_signing::{ChainId, WifSigner};
use std::sync::{mpsc, Arc};
use std::time::Duration;

fn live_client() -> RpcClient<HttpTransport> {
    RpcClient::new(HttpTransport::new(ACTA_TESTNET_HTTP_URL))
}

fn live_broadcast_client() -> RpcClient<GrapheneWebSocketApiTransport> {
    let session = Arc::new(
        GrapheneWebSocketSession::connect(ACTA_TESTNET_WS_URL)
            .expect("Acta WebSocket session should connect"),
    );
    let api = session
        .login_api("network_broadcast")
        .expect("network_broadcast API should log in");
    RpcClient::new(session.api_transport(&api))
}

#[test]
#[ignore = "requires network access and broadcasts a tiny Acta testnet transfer with callback"]
fn live_broadcasts_tiny_transfer_with_callback() {
    let signer =
        WifSigner::from_wif(PUBLIC_ACTA_TESTNET_WIF).expect("public Acta testnet WIF should parse");

    let client = live_client();
    let from = lookup_exact_account_id(&client, ACTA_TESTNET_FROM_ACCOUNT)
        .expect("from account fixture should exist");
    let to = lookup_exact_account_id(&client, ACTA_TESTNET_TO_ACCOUNT)
        .expect("to account fixture should exist");
    assert_ne!(from, to, "live transfer requires distinct accounts");

    let chain_id = client
        .call(GetChainIdParams)
        .expect("chain id should decode");
    let chain_id = ChainId::try_from(chain_id.as_str()).expect("chain id should parse");
    let prepared = prepare_transfer_transaction(
        &client,
        TransferDraft {
            from,
            to,
            amount: ACTA_TESTNET_TRANSFER_AMOUNT,
            asset_id: ACTA_TESTNET_TRANSFER_ASSET.to_owned(),
        },
        ACTA_TESTNET_TRANSFER_ASSET,
        chrono::Duration::minutes(5),
    )
    .expect("transfer transaction should prepare with fee and dynamic global properties");
    let signed = prepared
        .sign(&chain_id, &signer)
        .expect("transaction should sign");

    let session = GrapheneWebSocketSession::connect(ACTA_TESTNET_WS_URL)
        .expect("Acta WebSocket session should connect");
    let broadcast_api = session
        .login_api("network_broadcast")
        .expect("network_broadcast API should log in");
    let response =
        broadcast_signed_transaction_with_callback_typed(&session, &broadcast_api, signed)
            .expect("signed transfer should broadcast and receive callback");

    println!(
        "broadcast_transaction_with_callback => id={}, block_num={}, trx_num={}",
        response.id, response.block_num, response.trx_num
    );
    assert!(!response.id.is_empty());
}

#[test]
#[ignore = "requires network access and waits for an Acta testnet block notice"]
fn live_receives_block_applied_callback_notice() {
    let session = GrapheneWebSocketSession::connect(ACTA_TESTNET_WS_URL)
        .expect("Acta WebSocket session should connect");
    let database_api = session
        .login_api("database")
        .expect("database API should log in");

    let (notice_tx, notice_rx) = mpsc::sync_channel(1);
    let subscription = set_block_applied_callback(&session, &database_api, move |notice| {
        let _ = notice_tx.send(notice);
    })
    .expect("set_block_applied_callback should register callback");

    let notice = notice_rx
        .recv()
        .expect("registered callback should receive the notice payload");

    let block_id = notice.block_id;
    println!(
        "set_block_applied_callback => callback_id={}, block_id={block_id}",
        subscription.id()
    );
    assert_eq!(
        block_id.len(),
        40,
        "Graphene block ids should be 20-byte hex strings"
    );
    assert!(
        block_id
            .chars()
            .all(|character| character.is_ascii_hexdigit()),
        "Graphene block id should be hex"
    );

    subscription
        .unsubscribe()
        .expect("callback subscription should unsubscribe locally");
}

#[test]
#[ignore = "requires network access and waits for an Acta dynamic global properties notice"]
fn live_receives_dynamic_global_properties_subscription_notice() {
    let session = GrapheneWebSocketSession::connect(ACTA_TESTNET_WS_URL)
        .expect("Acta WebSocket session should connect");
    let database_api = session
        .login_api("database")
        .expect("database API should log in");

    let (dynamic_tx, dynamic_rx) = mpsc::sync_channel(1);
    let (subscription, initial) =
        subscribe_dynamic_global_properties(&session, &database_api, move |dynamic| {
            let _ = dynamic_tx.send(dynamic);
        })
        .expect("dynamic global properties subscription should register");

    println!(
        "subscribe_dynamic_global_properties initial => callback_id={}, head_block_number={}, witness={}",
        subscription.id(),
        initial.head_block_number,
        initial.current_witness.as_str()
    );
    assert!(initial.head_block_number > 0);
    assert!(initial.current_witness.as_str().starts_with("1.6."));

    let notice = dynamic_rx
        .recv_timeout(Duration::from_secs(20))
        .expect("registered subscription should receive a dynamic global properties notice");
    println!(
        "subscribe_dynamic_global_properties notice => callback_id={}, head_block_number={}, witness={}",
        subscription.id(),
        notice.head_block_number,
        notice.current_witness.as_str()
    );
    assert!(notice.head_block_number >= initial.head_block_number);
    assert!(notice.current_witness.as_str().starts_with("1.6."));

    subscription
        .unsubscribe()
        .expect("dynamic global properties subscription should unsubscribe locally");
}

#[test]
#[ignore = "requires network access and broadcasts a tiny Acta testnet transfer"]
fn live_signs_and_broadcasts_tiny_transfer_with_wif() {
    let signer =
        WifSigner::from_wif(PUBLIC_ACTA_TESTNET_WIF).expect("public Acta testnet WIF should parse");

    let client = live_client();
    let from = lookup_exact_account_id(&client, ACTA_TESTNET_FROM_ACCOUNT)
        .expect("from account fixture should exist");
    let to = lookup_exact_account_id(&client, ACTA_TESTNET_TO_ACCOUNT)
        .expect("to account fixture should exist");
    assert_ne!(from, to, "live transfer requires distinct accounts");

    let chain_id = client
        .call(GetChainIdParams)
        .expect("chain id should decode");
    let chain_id = ChainId::try_from(chain_id.as_str()).expect("chain id should parse");
    let prepared = prepare_transfer_transaction(
        &client,
        TransferDraft {
            from,
            to,
            amount: ACTA_TESTNET_TRANSFER_AMOUNT,
            asset_id: ACTA_TESTNET_TRANSFER_ASSET.to_owned(),
        },
        ACTA_TESTNET_TRANSFER_ASSET,
        chrono::Duration::minutes(5),
    )
    .expect("transfer transaction should prepare with fee and dynamic global properties");
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
