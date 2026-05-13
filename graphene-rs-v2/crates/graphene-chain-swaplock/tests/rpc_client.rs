use graphene_chain_swaplock::GetAccountCountParams;
use graphene_rpc::{GrapheneUInt64, RpcClient, RpcError, RpcTransport};
use serde_json::{json, Value};

struct MockTransport;

impl RpcTransport for MockTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        assert_eq!(method, "get_account_count");
        assert!(params.is_empty());
        Ok(json!(13))
    }
}

#[test]
fn generated_params_work_with_rpc_client() {
    let client = RpcClient::new(MockTransport);
    let response = client.call(GetAccountCountParams).unwrap();

    assert_eq!(response, GrapheneUInt64::new(13));
}
