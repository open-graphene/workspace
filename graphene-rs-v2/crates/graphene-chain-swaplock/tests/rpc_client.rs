use graphene_chain_swaplock::AboutParams;
use graphene_rpc::{RpcClient, RpcError, RpcTransport};
use serde_json::{json, Value};

struct MockTransport;

impl RpcTransport for MockTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        assert_eq!(method, "about");
        assert!(params.is_empty());
        Ok(json!({ "name": "swaplock" }))
    }
}

#[test]
fn generated_params_work_with_rpc_client() {
    let client = RpcClient::new(MockTransport);
    let response = client.call(AboutParams).unwrap();

    assert_eq!(response, json!({ "name": "swaplock" }));
}
