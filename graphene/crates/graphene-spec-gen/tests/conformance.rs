use graphene_spec_gen::conformance::{
    build_swaplock_transfer_fixture, BroadcastMethod, ConformanceFixturePhase,
};

#[test]
fn conformance_transfer_fixture_uses_rust_reference_paths() {
    let fixture = build_swaplock_transfer_fixture().expect("fixture should build from Rust paths");

    assert_eq!(fixture.schema_version, "1");
    assert_eq!(fixture.name, "swaplock-transfer");
    assert_eq!(
        fixture.chain_id,
        "2267f694d96b7ffdcba1a98c63c09e720a18a85ad34954e299c66d5a42234098"
    );
    assert_eq!(fixture.input.operation_name, "transfer");
    assert_eq!(fixture.input.operation_json["from"], "1.2.100");
    assert_eq!(fixture.input.operation_json["to"], "1.2.101");
    assert_eq!(fixture.input.transaction_json["operations"][0][0], 0);

    assert_eq!(
        fixture.expected.operation_hex,
        concat!(
            "00",
            "400d03000000000000",
            "6465",
            "393000000000000000",
            "0000"
        )
    );
    assert_eq!(
        fixture.expected.transaction_hex,
        concat!(
            "3412efcdab89",
            "00f15365",
            "01",
            "00",
            "400d03000000000000",
            "6465",
            "393000000000000000",
            "0000",
            "00"
        )
    );
    assert_eq!(
        fixture.expected.digest_hex,
        "6c664c46da08b6dcbd0cdf9e4f24dc051afaa6096a23db23b15377377c60805e"
    );
    assert_eq!(
        fixture.expected.public_key,
        "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
    );
    assert_eq!(fixture.expected.signature_hex.len(), 130);
    assert!(matches!(
        fixture.expected.signature_hex.get(..2),
        Some("1f" | "20" | "21" | "22")
    ));
    assert!(fixture.expected.signature_metadata.canonical);
    assert_eq!(
        fixture.expected.broadcast_payload.method,
        BroadcastMethod::BroadcastTransaction
    );
    assert_eq!(
        fixture.expected.broadcast_payload.params.api_name,
        "network_broadcast"
    );
    assert_eq!(
        fixture.expected.broadcast_payload.params.transaction_json["signatures"][0],
        fixture.expected.signature_hex
    );
}

#[test]
fn conformance_fixture_error_display_names_failed_phase() {
    let phase = ConformanceFixturePhase::OperationEncoding;
    let text = format!("{phase:?}");

    assert_eq!(text, "OperationEncoding");
}
