use fabric::authoring::{
    ComponentAugmentationDefinition, ResourceAugmentationDefinition, SystemAugmentationDefinition,
};
use third_party_base_semantics::{ThirdPartyClock, ThirdPartyComponent, ThirdPartyStore};

#[derive(Clone)]
pub struct ResourceAugmentation;
#[derive(Clone)]
pub struct ResourceAugmentationService;
impl ResourceAugmentationDefinition<ThirdPartyStore> for ResourceAugmentation {
    type Config = ();
    type Contract = ResourceAugmentationService;
    fn contract_key() -> fabric::core::ContractKey<Self::Contract> {
        fabric::core::ContractKey::provisional(
            fabric::core::ContractId::new("third.party.store.augmentation").expect("static id"),
        )
    }
}

#[derive(Clone)]
pub struct SystemAugmentation;
#[derive(Clone)]
pub struct SystemAugmentationService;
impl SystemAugmentationDefinition<ThirdPartyClock> for SystemAugmentation {
    type Config = ();
    type Contract = SystemAugmentationService;
    fn contract_key() -> fabric::core::ContractKey<Self::Contract> {
        fabric::core::ContractKey::provisional(
            fabric::core::ContractId::new("third.party.clock.augmentation").expect("static id"),
        )
    }
}

#[derive(Clone)]
pub struct ComponentX;
#[derive(Clone)]
pub struct ComponentXService;
impl ComponentAugmentationDefinition<ThirdPartyComponent> for ComponentX {
    type Config = ();
    type Contract = ComponentXService;
    fn contract_key() -> fabric::core::ContractKey<Self::Contract> {
        fabric::core::ContractKey::provisional(
            fabric::core::ContractId::new("third.party.component.x").expect("static id"),
        )
    }
}

#[derive(Clone)]
pub struct ComponentY;
#[derive(Clone)]
pub struct ComponentYService;
impl ComponentAugmentationDefinition<ThirdPartyComponent> for ComponentY {
    type Config = ();
    type Contract = ComponentYService;
    fn contract_key() -> fabric::core::ContractKey<Self::Contract> {
        fabric::core::ContractKey::provisional(
            fabric::core::ContractId::new("third.party.component.y").expect("static id"),
        )
    }
}
