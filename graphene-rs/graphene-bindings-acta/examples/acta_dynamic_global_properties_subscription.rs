use graphene_bindings_acta::{subscribe_dynamic_global_properties, ACTA_TESTNET_WS_URL};
use graphene_rpc::GrapheneWebSocketSession;
use std::sync::mpsc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = GrapheneWebSocketSession::connect(ACTA_TESTNET_WS_URL)?;
    let database_api = session.login_api("database")?;

    let (dynamic_tx, dynamic_rx) = mpsc::sync_channel(1);
    let (subscription, initial) =
        subscribe_dynamic_global_properties(&session, &database_api, move |dynamic| {
            let _ = dynamic_tx.send(dynamic);
        })?;

    println!(
        "initial dynamic global properties => head_block_number={}, witness={}",
        initial.head_block_number,
        initial.current_witness.as_str()
    );

    let notice = dynamic_rx.recv_timeout(Duration::from_secs(20))?;
    println!(
        "dynamic global properties notice => callback_id={}, head_block_number={}, witness={}",
        subscription.id(),
        notice.head_block_number,
        notice.current_witness.as_str()
    );

    subscription.unsubscribe()?;
    Ok(())
}
