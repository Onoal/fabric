use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use fabric::prelude::*;
use fabric::{
    ResourceAugmentation as ResourceAttachment, SystemAugmentation as SystemAttachment,
    component::ComponentRuntimeScope,
    core::{
        ModuleBindings, ModuleContract, ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
    },
};
use third_party_augmentation_semantics::{
    ComponentX, ComponentXService, ComponentY, ComponentYService, ResourceAugmentation,
    ResourceAugmentationService, SystemAugmentation, SystemAugmentationService,
};
use third_party_base_semantics::{ThirdPartyClock, ThirdPartyComponent, ThirdPartyStore};

macro_rules! runtime {
    ($name:ident, $semantic:ident, $service:expr) => {
        struct $name {
            id: ModuleId,
        }
        impl ModuleRuntime for $name {
            fn id(&self) -> &ModuleId {
                &self.id
            }
            fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
                Ok(vec![ModuleContract::new(
                    &$semantic::contract_key(),
                    Arc::new($service),
                )])
            }
            fn bind(&mut self, _: &ModuleBindings) -> Result<(), ModuleError> {
                Ok(())
            }
            fn initialize(&mut self) -> Result<(), ModuleError> {
                Ok(())
            }
            fn start(&mut self) -> Result<(), ModuleError> {
                Ok(())
            }
            fn stop(&mut self) -> Result<(), ModuleError> { Ok(()) }
            fn health(&self) -> Health {
                Health::Healthy
            }
        }
    };
}

runtime!(
    ResourceRuntime,
    ResourceAugmentation,
    ResourceAugmentationService
);
runtime!(SystemRuntime, SystemAugmentation, SystemAugmentationService);
runtime!(ComponentXRuntime, ComponentX, ComponentXService);
runtime!(ComponentYRuntime, ComponentY, ComponentYService);

#[derive(Clone)]
pub struct ResourceSupport;
impl ResourceAugmentationSupportDefinition<ThirdPartyStore, ResourceAugmentation>
    for ResourceSupport
{
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(
        &self,
        _: &ResourceAttachment<ThirdPartyStore, ResourceAugmentation>,
        id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(ResourceRuntime { id }))
    }
}

#[derive(Clone)]
pub struct SystemSupport;
impl SystemAugmentationSupportDefinition<ThirdPartyClock, SystemAugmentation> for SystemSupport {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(
        &self,
        _: &SystemAttachment<ThirdPartyClock, SystemAugmentation>,
        id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(SystemRuntime { id }))
    }
}

#[derive(Clone)]
pub struct ComponentXSupport(pub Arc<AtomicUsize>);
impl ComponentAugmentationSupportDefinition<ThirdPartyComponent, ComponentX> for ComponentXSupport {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(&self, _: &(), id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(ComponentXRuntime { id }))
    }
    fn prepare(&self, _: &(), _: &ComponentRuntimeScope) -> Result<(), ComponentError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone)]
pub struct ComponentYSupport(pub Arc<AtomicUsize>);
impl ComponentAugmentationSupportDefinition<ThirdPartyComponent, ComponentY> for ComponentYSupport {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(&self, _: &(), id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(ComponentYRuntime { id }))
    }
    fn prepare(&self, _: &(), _: &ComponentRuntimeScope) -> Result<(), ComponentError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
