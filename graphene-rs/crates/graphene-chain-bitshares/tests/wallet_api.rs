use graphene_chain_bitshares::wallet_api::{
    BITSHARES_WALLET_API_METHODS, BlindTransferParams, InfoParams, SignBuilderTransaction2Params,
    WalletApiParams,
};

fn method(name: &str) -> &'static graphene_chain_bitshares::wallet_api::WalletApiMethodSpec {
    BITSHARES_WALLET_API_METHODS
        .iter()
        .find(|method| method.name == name)
        .unwrap_or_else(|| panic!("missing generated wallet API method {name}"))
}

#[test]
fn generated_wallet_api_contains_public_fc_api_surface() {
    assert_eq!(BITSHARES_WALLET_API_METHODS.len(), 144);

    for name in [
        "info",
        "transfer",
        "sell_asset",
        "sign_transaction",
        "broadcast_transaction",
        "account_store_map",
    ] {
        assert!(
            BITSHARES_WALLET_API_METHODS
                .iter()
                .any(|method| method.name == name),
            "missing public wallet API method {name}"
        );
    }
}

#[test]
fn generated_wallet_api_excludes_non_fc_api_helpers() {
    for name in [
        "copy_wallet_file",
        "derive_private_key",
        "blind_transfer_help",
    ] {
        assert!(
            !BITSHARES_WALLET_API_METHODS
                .iter()
                .any(|method| method.name == name),
            "helper method {name} should not be part of generated public wallet API"
        );
    }
}

#[test]
fn generated_wallet_api_preserves_transfer_signature() {
    let transfer = method("transfer");

    assert_eq!(transfer.return_cpp_type, "signed_transaction");
    assert!(transfer.is_const);
    assert_eq!(
        transfer
            .params
            .iter()
            .map(|param| (param.name, param.cpp_type, param.default_value))
            .collect::<Vec<_>>(),
        vec![
            ("from", "const string&", None),
            ("to", "const string&", None),
            ("amount", "const string&", None),
            ("asset_symbol_or_id", "const string&", None),
            ("memo", "const string&", None),
            ("broadcast", "bool", Some("false")),
        ]
    );
    assert!(
        transfer
            .doc
            .contains("Transfer an amount from one account to another")
    );
}

#[test]
fn generated_wallet_api_preserves_complex_defaults() {
    let sign_builder_transaction2 = method("sign_builder_transaction2");

    assert_eq!(
        sign_builder_transaction2
            .params
            .iter()
            .map(|param| (param.name, param.cpp_type, param.default_value))
            .collect::<Vec<_>>(),
        vec![
            ("transaction_handle", "transaction_handle_type", None),
            (
                "signing_keys",
                "const vector<public_key_type>&",
                Some("vector<public_key_type>()"),
            ),
            ("broadcast", "bool", Some("true")),
        ]
    );
}

#[test]
fn generated_wallet_api_emits_raw_params_structs_with_positional_json() {
    let params = BlindTransferParams {
        from_key_or_label: "from-blind".to_owned(),
        to_key_or_label: "to-blind".to_owned(),
        amount: "1.00000".to_owned(),
        symbol_or_id: "BTS".to_owned(),
        broadcast: false,
    };

    assert_eq!(BlindTransferParams::METHOD, "blind_transfer");
    assert_eq!(BlindTransferParams::RETURN_CPP_TYPE, "blind_confirmation");
    assert_eq!(
        params.into_positional_params(),
        vec![
            serde_json::json!("from-blind"),
            serde_json::json!("to-blind"),
            serde_json::json!("1.00000"),
            serde_json::json!("BTS"),
            serde_json::json!(false),
        ]
    );
}

#[test]
fn generated_wallet_api_emits_empty_and_vector_param_shapes() {
    assert_eq!(InfoParams::METHOD, "info");
    assert_eq!(
        InfoParams.into_positional_params(),
        Vec::<serde_json::Value>::new()
    );

    let params = SignBuilderTransaction2Params {
        transaction_handle: 7,
        signing_keys: vec!["BTS6MRyAjQq8ud7hVNYcfnVPJqcVpscN5SoM7wPCCnM".to_owned()],
        broadcast: true,
    };

    assert_eq!(
        SignBuilderTransaction2Params::METHOD,
        "sign_builder_transaction2"
    );
    assert_eq!(
        params.into_positional_params(),
        vec![
            serde_json::json!(7),
            serde_json::json!(["BTS6MRyAjQq8ud7hVNYcfnVPJqcVpscN5SoM7wPCCnM"]),
            serde_json::json!(true),
        ]
    );
}
