use graphene_chain_bitshares::types::account;
use serde_json::{Value, json};
use std::process::Command;

#[test]
#[ignore = "requires live BitShares RPC endpoint"]
fn deserializes_account_object_from_live_rpc() {
    let endpoint = std::env::var("BITSHARES_RPC_ENDPOINT")
        .unwrap_or_else(|_| "https://cloud.xbts.io/ws".to_owned());
    let endpoint = endpoint
        .strip_prefix("wss://")
        .map(|host| format!("https://{host}"))
        .unwrap_or(endpoint);

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "call",
        "params": [0, "get_objects", [["1.2.0"]]]
    });

    let output = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            "20",
            "--header",
            "content-type: application/json",
            "--data",
            &request.to_string(),
            &endpoint,
        ])
        .output()
        .expect("curl should be available to call the BitShares RPC endpoint");

    assert!(
        output.status.success(),
        "curl failed with status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let response: Value = serde_json::from_slice(&output.stdout)
        .expect("BitShares RPC response should be valid JSON");
    assert!(
        response.get("error").is_none(),
        "BitShares RPC returned error: {response}"
    );

    let account_value = response
        .get("result")
        .and_then(Value::as_array)
        .and_then(|objects| objects.first())
        .cloned()
        .expect("get_objects should return the committee account object");

    let account: account::Object = serde_json::from_value(account_value)
        .expect("live committee account JSON should deserialize into account::Object");

    assert_eq!(account.id.to_string(), "1.2.0");
    assert_eq!(account.name, "committee-account");
    assert_eq!(account.registrar.to_string(), "1.2.0");
    assert_eq!(account.options.voting_account.to_string(), "1.2.5");
    assert!(matches!(
        account.owner_special_authority,
        graphene_protocol::SpecialAuthority::None(_)
    ));
    assert!(matches!(
        account.active_special_authority,
        graphene_protocol::SpecialAuthority::None(_)
    ));
}
