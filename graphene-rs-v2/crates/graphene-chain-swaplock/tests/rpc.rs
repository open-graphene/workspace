use graphene_chain_swaplock::{SignedTransaction, TransferParams};
use graphene_rpc::OpenRpcParams;
use serde_json::json;

#[test]
fn transfer_params_emit_method_response_and_positional_params() {
    fn assert_response_type<T: OpenRpcParams<Response = SignedTransaction>>() {}
    assert_response_type::<TransferParams>();

    let params = TransferParams {
        r#from: "alice".to_owned(),
        to: "bob".to_owned(),
        amount: "1.00000".to_owned(),
        asset_symbol_or_id: "BTS".to_owned(),
        memo: "hello".to_owned(),
        broadcast: false,
    };

    assert_eq!(TransferParams::METHOD, "transfer");
    assert_eq!(
        params.into_positional_params(),
        vec![
            json!("alice"),
            json!("bob"),
            json!("1.00000"),
            json!("BTS"),
            json!("hello"),
            json!(false),
        ]
    );
}
