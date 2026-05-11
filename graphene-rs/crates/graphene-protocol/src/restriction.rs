/// JSON payload of a custom-authority restriction argument.
///
/// The argument is a large Graphene `static_variant`; keep the raw JSON value until operation
/// argument variants are modeled explicitly.
#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
pub struct RestrictionArgument(pub serde_json::Value);

impl Eq for RestrictionArgument {}

/// Custom-authority restriction as exposed by Graphene JSON.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct Restriction {
    pub member_index: u64,
    pub restriction_type: u64,
    pub argument: RestrictionArgument,
    pub extensions: Vec<RestrictionArgument>,
}
