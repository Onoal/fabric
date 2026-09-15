use fabric_core::ContractKey;

use super::SystemDefinition;

pub trait PrimarySystemContract: SystemDefinition {
    type Contract: Clone + Send + Sync + 'static;

    fn primary_contract_key() -> ContractKey<Self::Contract>;
}
