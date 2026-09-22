use fabric_core::ContractRequirement;

use super::SystemDefinition;

pub trait AdaptableSystemDefinition: SystemDefinition {
    type RealizationContract: Send + Sync + 'static;

    fn realization_requirement() -> ContractRequirement<Self::RealizationContract>;

    /// See [`crate::authoring::AdaptableResourceDefinition::supports_semantic_api_adapter`].
    #[doc(hidden)]
    fn supports_semantic_api_adapter() -> bool {
        false
    }
}
