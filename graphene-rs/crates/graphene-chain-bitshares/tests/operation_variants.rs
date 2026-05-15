use graphene_chain_bitshares::operation_variants::BITSHARES_OPERATION_VARIANTS;

#[test]
fn bitshares_operation_variants_match_declaration_order_tags() {
    assert_eq!(BITSHARES_OPERATION_VARIANTS.len(), 78);

    for (expected_tag, variant) in BITSHARES_OPERATION_VARIANTS.iter().enumerate() {
        assert_eq!(
            variant.tag as usize, expected_tag,
            "operation tag table should be contiguous at index {expected_tag}"
        );
    }

    assert_eq!(
        BITSHARES_OPERATION_VARIANTS[0].cpp_type,
        "transfer_operation"
    );
    assert_eq!(
        BITSHARES_OPERATION_VARIANTS[37].cpp_type,
        "balance_claim_operation"
    );
    assert_eq!(
        BITSHARES_OPERATION_VARIANTS[77].cpp_type,
        "limit_order_update_operation"
    );
}

#[test]
fn bitshares_operation_variants_preserve_virtual_markers() {
    assert!(
        BITSHARES_OPERATION_VARIANTS
            .iter()
            .any(|variant| variant.is_virtual),
        "expected generated table to preserve at least one // VIRTUAL operation marker"
    );

    let fill_order = BITSHARES_OPERATION_VARIANTS
        .iter()
        .find(|variant| variant.cpp_type == "fill_order_operation")
        .expect("fill_order_operation should be present");
    assert_eq!(fill_order.tag, 4);
    assert!(fill_order.is_virtual);
}
