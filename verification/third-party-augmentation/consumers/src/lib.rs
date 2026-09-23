use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use fabric::authoring::{
    ComponentAugmentationRequirement, ResourceAugmentationRequirement,
    SystemAugmentationRequirement,
};
use fabric::core::{ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime};
use fabric::prelude::*;
use third_party_augmentation_semantics::{
    ComponentX, ComponentY, ResourceAugmentation, SystemAugmentation,
};
use third_party_base_semantics::{ThirdPartyClock, ThirdPartyComponent, ThirdPartyStore};

macro_rules! simple_component_consumer {
    ($name:ident, $semantic:ty, $service:ty, $id:literal) => {
        #[derive(Clone)]
        pub struct $name {
            id: ModuleId,
            requirement: ComponentAugmentationRequirement<ThirdPartyComponent, $semantic>,
            bound: Arc<AtomicUsize>,
        }
        impl $name {
            pub fn new(
                requirement: ComponentAugmentationRequirement<ThirdPartyComponent, $semantic>,
                bound: Arc<AtomicUsize>,
            ) -> Self {
                Self {
                    id: ModuleId::new($id).expect("static module id"),
                    requirement,
                    bound,
                }
            }
            pub fn provider_selection(&self) -> fabric::core::ContractProviderSelection {
                self.requirement.provider_selection(self.id.clone())
            }
        }
        impl ModuleRuntime for $name {
            fn id(&self) -> &ModuleId {
                &self.id
            }
            fn required_contract_declarations(
                &self,
            ) -> Vec<fabric::core::ContractRequirementDeclaration> {
                vec![self.requirement.augmentation().declaration().clone()]
            }
            fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
                Ok(Vec::new())
            }
            fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
                bindings
                    .resolve(self.requirement.augmentation())
                    .map_err(|error| ModuleError::new(error.to_string()))?;
                self.bound.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            fn initialize(&mut self) -> Result<(), ModuleError> {
                Ok(())
            }
            fn start(&mut self) -> Result<(), ModuleError> {
                Ok(())
            }
            fn stop(&mut self) -> Result<(), ModuleError> {
                Ok(())
            }
            fn health(&self) -> Health {
                Health::Healthy
            }
        }
    };
}

#[derive(Clone)]
pub struct ResourceConsumer {
    id: ModuleId,
    requirement: Arc<ResourceAugmentationRequirement<ThirdPartyStore, ResourceAugmentation>>,
    bound: Arc<AtomicUsize>,
}
impl ResourceConsumer {
    pub fn new(
        requirement: ResourceAugmentationRequirement<ThirdPartyStore, ResourceAugmentation>,
        bound: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            id: ModuleId::new("third.party.resource.consumer").expect("static module id"),
            requirement: Arc::new(requirement),
            bound,
        }
    }
    pub fn provider_selections(&self) -> [fabric::core::ContractProviderSelection; 2] {
        self.requirement.provider_selections(self.id.clone())
    }
}
impl ModuleRuntime for ResourceConsumer {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    fn required_contract_declarations(&self) -> Vec<fabric::core::ContractRequirementDeclaration> {
        vec![
            self.requirement.base().declaration().clone(),
            self.requirement.augmentation().declaration().clone(),
        ]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.requirement
            .base()
            .resolve(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        bindings
            .resolve(self.requirement.augmentation())
            .map_err(|error| ModuleError::new(error.to_string()))?;
        self.bound.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone)]
pub struct SystemConsumer {
    id: ModuleId,
    requirement: Arc<SystemAugmentationRequirement<ThirdPartyClock, SystemAugmentation>>,
    bound: Arc<AtomicUsize>,
}
impl SystemConsumer {
    pub fn new(
        requirement: SystemAugmentationRequirement<ThirdPartyClock, SystemAugmentation>,
        bound: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            id: ModuleId::new("third.party.system.consumer").expect("static module id"),
            requirement: Arc::new(requirement),
            bound,
        }
    }
    pub fn provider_selections(&self) -> [fabric::core::ContractProviderSelection; 2] {
        self.requirement.provider_selections(self.id.clone())
    }
}
impl ModuleRuntime for SystemConsumer {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    fn required_contract_declarations(&self) -> Vec<fabric::core::ContractRequirementDeclaration> {
        vec![
            self.requirement.base().declaration().clone(),
            self.requirement.augmentation().declaration().clone(),
        ]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.requirement
            .base()
            .resolve(bindings)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        bindings
            .resolve(self.requirement.augmentation())
            .map_err(|error| ModuleError::new(error.to_string()))?;
        self.bound.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn health(&self) -> Health {
        Health::Healthy
    }
}

simple_component_consumer!(
    ComponentXConsumer,
    ComponentX,
    ComponentXService,
    "third.party.component.x.consumer"
);
simple_component_consumer!(
    ComponentYConsumer,
    ComponentY,
    ComponentYService,
    "third.party.component.y.consumer"
);
