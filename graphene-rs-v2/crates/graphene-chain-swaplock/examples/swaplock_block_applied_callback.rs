use graphene_chain_swaplock::{set_block_applied_callback, SWAPLOCK_TESTNET_WS_URL};
use graphene_rpc::GrapheneWebSocketSession;
use std::sync::mpsc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = GrapheneWebSocketSession::connect(SWAPLOCK_TESTNET_WS_URL)?;
    let database_api = session.login_api("database")?;

    let (notice_tx, notice_rx) = mpsc::sync_channel(1);
    let subscription = set_block_applied_callback(&session, &database_api, move |notice| {
        let _ = notice_tx.send(notice);
    })?;

    println!(
        "registered set_block_applied_callback with callback_id={}",
        subscription.id()
    );
    println!("waiting for the next block notice...");

    let notice = notice_rx.recv()?;
    println!(
        "set_block_applied_callback => callback_id={}, block_id={}",
        subscription.id(),
        notice.block_id
    );

    subscription.unsubscribe()?;
    session.shutdown()?;
    Ok(())
}
