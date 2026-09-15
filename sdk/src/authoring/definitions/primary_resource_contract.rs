use fabric_core::ContractKey;

use super::ResourceDefinition;

pub trait PrimaryResourceContract: ResourceDefinition {
    type Contract: Clone + Send + Sync + 'static;

    fn primary_contract_key() -> ContractKey<Self::Contract>;
}
