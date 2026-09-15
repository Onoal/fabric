use fabric_core::{
    HostMaterializationRequirement, Module, ModuleDeclaration, ModuleId, ModuleRuntime,
};

use super::AdapterDefinition;

pub struct AdapterProviderModule<A>
where
    A: AdapterDefinition,
{
    provider_module_id: ModuleId,
    adapter: A,
}

impl<A> Clone for AdapterProviderModule<A>
where
    A: AdapterDefinition,
{
    fn clone(&self) -> Self {
        Self {
            provider_module_id: self.provider_module_id.clone(),
            adapter: self.adapter.clone(),
        }
    }
}

impl<A> AdapterProviderModule<A>
where
    A: AdapterDefinition,
{
    pub(crate) fn new(provider_module_id: ModuleId, adapter: A) -> Self {
        Self {
            provider_module_id,
            adapter,
        }
    }

    pub fn provider_module_id(&self) -> &ModuleId {
        &self.provider_module_id
    }

    pub fn adapter(&self) -> &A {
        &self.adapter
    }
}

impl<A> Module for AdapterProviderModule<A>
where
    A: AdapterDefinition,
{
    fn declaration(&self) -> ModuleDeclaration {
        self.adapter
            .declaration(self.provider_module_id.clone())
            .with_host_requirement(HostMaterializationRequirement::new(
                self.provider_module_id.clone(),
                self.adapter.host_requirement(),
            ))
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        self.adapter
            .materialize_provider(self.provider_module_id.clone())
    }
}
