/// One configured FC-reflected object whose Rust `Object` struct should be generated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectGeneration {
    /// Object family module name, for example `account_balance`.
    pub family_name: &'static str,
    /// C++ class name without namespace, for example `account_balance_object`.
    pub cpp_class: &'static str,
    /// Header path relative to a chain core root.
    pub header_path: &'static str,
    /// C++ source path containing FC reflection, relative to a chain core root.
    pub source_path: &'static str,
}

/// Reflected chain objects currently proven by the generator and fixture tests.
pub const GENERATED_OBJECTS: &[ObjectGeneration] = &[
    ObjectGeneration {
        family_name: "account_balance",
        cpp_class: "account_balance_object",
        header_path: "libraries/chain/include/graphene/chain/account_object.hpp",
        source_path: "libraries/chain/account_object.cpp",
    },
    ObjectGeneration {
        family_name: "account_history",
        cpp_class: "account_history_object",
        header_path: "libraries/chain/include/graphene/chain/operation_history_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "asset_dynamic_data",
        cpp_class: "asset_dynamic_data_object",
        header_path: "libraries/chain/include/graphene/chain/asset_object.hpp",
        source_path: "libraries/chain/asset_object.cpp",
    },
    ObjectGeneration {
        family_name: "blinded_balance",
        cpp_class: "blinded_balance_object",
        header_path: "libraries/chain/include/graphene/chain/confidential_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "block_summary",
        cpp_class: "block_summary_object",
        header_path: "libraries/chain/include/graphene/chain/block_summary_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "buyback",
        cpp_class: "buyback_object",
        header_path: "libraries/chain/include/graphene/chain/buyback_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "chain_property",
        cpp_class: "chain_property_object",
        header_path: "libraries/chain/include/graphene/chain/chain_property_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "committee_member",
        cpp_class: "committee_member_object",
        header_path: "libraries/chain/include/graphene/chain/committee_member_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "dynamic_global_property",
        cpp_class: "dynamic_global_property_object",
        header_path: "libraries/chain/include/graphene/chain/global_property_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "fba_accumulator",
        cpp_class: "fba_accumulator_object",
        header_path: "libraries/chain/include/graphene/chain/fba_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "witness",
        cpp_class: "witness_object",
        header_path: "libraries/chain/include/graphene/chain/witness_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "witness_schedule",
        cpp_class: "witness_schedule_object",
        header_path: "libraries/chain/include/graphene/chain/witness_schedule_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
    ObjectGeneration {
        family_name: "withdraw_permission",
        cpp_class: "withdraw_permission_object",
        header_path: "libraries/chain/include/graphene/chain/withdraw_permission_object.hpp",
        source_path: "libraries/chain/small_objects.cpp",
    },
];
