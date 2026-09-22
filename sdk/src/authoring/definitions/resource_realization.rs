use fabric_core::ContractProviderSelection;

use super::{
    AdaptableResourceDefinition, AdapterBridgeMode, AdapterDefinition, AdapterProviderModule,
    ResourceAdapterCompatibility, ResourceSelection,
};

pub struct ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R>,
    A::Compatibility: ResourceAdapterCompatibility<R>,
{
    resource: ResourceSelection<R>,
    adapter: AdapterProviderModule<A>,
    selection: Option<ContractProviderSelection>,
    bridge_mode: AdapterBridgeMode,
}

impl<R, A> Clone for ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R>,
    A::Compatibility: ResourceAdapterCompatibility<R>,
{
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
            adapter: self.adapter.clone(),
            selection: self.selection.clone(),
            bridge_mode: self.bridge_mode,
        }
    }
}

impl<R, A> ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R>,
    A::Compatibility: ResourceAdapterCompatibility<R>,
{
    pub(crate) fn new(
        resource: ResourceSelection<R>,
        adapter: AdapterProviderModule<A>,
        selection: Option<ContractProviderSelection>,
        bridge_mode: AdapterBridgeMode,
    ) -> Self {
        Self {
            resource,
            adapter,
            selection,
            bridge_mode,
        }
    }

    pub fn resource(&self) -> &ResourceSelection<R> {
        &self.resource
    }

    pub fn adapter(&self) -> &AdapterProviderModule<A> {
        &self.adapter
    }

    pub fn provider_selection(&self) -> &ContractProviderSelection {
        self.selection
            .as_ref()
            .expect("canonical semantic API adapters select their target directly")
    }

    pub fn into_raw_parts(
        self,
    ) -> (
        ResourceSelection<R>,
        AdapterProviderModule<A>,
        ContractProviderSelection,
    ) {
        (
            self.resource,
            self.adapter,
            self.selection
                .expect("into_raw_parts is the legacy explicit-realization boundary"),
        )
    }

    pub(crate) fn into_bridge_parts(
        self,
    ) -> (
        ResourceSelection<R>,
        AdapterProviderModule<A>,
        Option<ContractProviderSelection>,
        AdapterBridgeMode,
    ) {
        (
            self.resource,
            self.adapter,
            self.selection,
            self.bridge_mode,
        )
    }
}
