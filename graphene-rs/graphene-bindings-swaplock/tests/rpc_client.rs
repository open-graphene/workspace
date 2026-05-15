use graphene_bindings_swaplock::{lookup_exact_account_id, GetAccountCountParams};
use graphene_rpc::{GrapheneUInt64, RpcClient, RpcError, RpcTransport};
use serde_json::{json, Value};

struct MockTransport;
struct AccountLookupTransport;

impl RpcTransport for MockTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        assert_eq!(method, "get_account_count");
        assert!(params.is_empty());
        Ok(json!(13))
    }
}

impl RpcTransport for AccountLookupTransport {
    fn call_raw(&self, method: &str, params: Vec<Value>) -> Result<Value, RpcError> {
        assert_eq!(method, "lookup_accounts");
        assert_eq!(params, vec![json!("swaplock"), json!(1), json!(false)]);
        Ok(json!([["swaplock", "1.2.100"]]))
    }
}

#[test]
fn generated_params_work_with_rpc_client() {
    let client = RpcClient::new(MockTransport);
    let response = client.call(GetAccountCountParams).unwrap();

    assert_eq!(response, GrapheneUInt64::new(13));
}

#[test]
fn lookup_exact_account_id_uses_lookup_accounts_with_exact_match() {
    let client = RpcClient::new(AccountLookupTransport);
    let account_id = lookup_exact_account_id(&client, "swaplock").unwrap();

    assert_eq!(account_id, "1.2.100");
}
