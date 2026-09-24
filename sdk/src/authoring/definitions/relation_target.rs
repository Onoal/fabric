use fabric_core::{ContractRequirement, ContractVersionRequirement};
use fabric_resource::ResourceId;
use fabric_system::SystemId;

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationTargetDescriptor {
    Resource(ResourceId),
    System(SystemId),
}

/// A semantic definition whose primary contract can be required by another
/// Fabric definition.
///
/// This is authoring/runtime machinery, not a third semantic subject: Resource
/// and System retain their own identities and realization models.
pub trait RelationTarget: Send + Sync + 'static {
    type Contract: Clone + Send + Sync + 'static;

    #[doc(hidden)]
    fn relation_target_descriptor() -> RelationTargetDescriptor;

    fn relation_requirement() -> ContractRequirement<Self::Contract>;

    fn relation_requirement_versioned(
        requirement: ContractVersionRequirement,
    ) -> ContractRequirement<Self::Contract>;
}
