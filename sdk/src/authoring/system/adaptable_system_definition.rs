use fabric_core::ContractRequirement;

use super::SystemDefinition;

pub trait AdaptableSystemDefinition: SystemDefinition {
    type RealizationContract: Send + Sync + 'static;

    fn realization_requirement() -> ContractRequirement<Self::RealizationContract>;
}
