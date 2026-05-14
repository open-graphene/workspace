use graphene_chain_swaplock::SWAPLOCK_TESTNET_WS_URL;
use graphene_rpc::GrapheneWebSocketSession;
use std::sync::mpsc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = GrapheneWebSocketSession::connect(SWAPLOCK_TESTNET_WS_URL)?;
    let database_api = session.login_api("database")?;

    let (payload_tx, payload_rx) = mpsc::sync_channel(1);
    let (subscription, response) = session.call_with_callback_raw(
        &database_api,
        "set_block_applied_callback",
        Vec::new(),
        move |payload| {
            let _ = payload_tx.send(payload);
        },
    )?;

    if !response.is_null() {
        return Err(format!(
            "set_block_applied_callback should acknowledge with null, got {response:?}"
        )
        .into());
    }

    println!(
        "registered set_block_applied_callback with callback_id={}",
        subscription.id()
    );
    println!("waiting for the next block notice...");

    let payload = payload_rx.recv()?;
    let block_id = payload
        .as_array()
        .and_then(|values| values.first())
        .and_then(serde_json::Value::as_str)
        .or_else(|| payload.as_str())
        .ok_or("block applied callback payload should contain a block id string")?;

    println!(
        "set_block_applied_callback => callback_id={}, block_id={block_id}",
        subscription.id()
    );

    subscription.unsubscribe()?;
    session.shutdown()?;
    Ok(())
}
