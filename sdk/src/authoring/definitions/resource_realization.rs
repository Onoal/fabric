use fabric_resource::AdapterResourceSchemaSupport;

use fabric_core::ContractProviderSelection;

use super::{
    AdaptableResourceDefinition, AdapterDefinition, AdapterProviderModule, ResourceSelection,
};

pub struct ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R, SchemaSupport = AdapterResourceSchemaSupport>,
{
    resource: ResourceSelection<R>,
    adapter: AdapterProviderModule<A>,
    selection: ContractProviderSelection,
}

impl<R, A> Clone for ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R, SchemaSupport = AdapterResourceSchemaSupport>,
{
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            adapter: self.adapter.clone(),
            selection: self.selection.clone(),
        }
    }
}

impl<R, A> ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R, SchemaSupport = AdapterResourceSchemaSupport>,
{
    pub(crate) fn new(
        resource: ResourceSelection<R>,
        adapter: AdapterProviderModule<A>,
        selection: ContractProviderSelection,
    ) -> Self {
        Self {
            resource,
            adapter,
            selection,
        }
    }

    pub fn resource(&self) -> &ResourceSelection<R> {
        &self.resource
    }

    pub fn adapter(&self) -> &AdapterProviderModule<A> {
        &self.adapter
    }

    pub fn provider_selection(&self) -> &ContractProviderSelection {
        &self.selection
    }

    pub fn into_raw_parts(
        self,
    ) -> (
        ResourceSelection<R>,
        AdapterProviderModule<A>,
        ContractProviderSelection,
    ) {
        (self.resource, self.adapter, self.selection)
    }
}
