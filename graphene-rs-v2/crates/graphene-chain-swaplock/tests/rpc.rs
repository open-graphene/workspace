use graphene_chain_swaplock::{GetBlockParams, SignedBlock};
use graphene_rpc::OpenRpcParams;
use serde_json::json;

#[test]
fn get_block_params_emit_method_response_and_positional_params() {
    fn assert_response_type<T: OpenRpcParams<Response = Option<SignedBlock>>>() {}
    assert_response_type::<GetBlockParams>();

    let params = GetBlockParams { block_num: 123 };

    assert_eq!(GetBlockParams::METHOD, "get_block");
    assert_eq!(params.into_positional_params(), vec![json!(123)]);
}
