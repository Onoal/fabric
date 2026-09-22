use fabric_core::{ContractRequirement, ContractVersionRequirement};

/// A semantic definition whose primary contract can be required by another
/// Fabric definition.
///
/// This is authoring/runtime machinery, not a third semantic subject: Resource
/// and System retain their own identities and realization models.
pub trait RelationTarget: Send + Sync + 'static {
    type Contract: Clone + Send + Sync + 'static;

    fn relation_requirement() -> ContractRequirement<Self::Contract>;

    fn relation_requirement_versioned(
        requirement: ContractVersionRequirement,
    ) -> ContractRequirement<Self::Contract>;
}
