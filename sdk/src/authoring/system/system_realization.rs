use fabric_core::ContractProviderSelection;

use super::{AdaptableSystemDefinition, SystemSelection};
use crate::authoring::definitions::{
    AdapterBridgeMode, AdapterDefinition, AdapterProviderModule, SystemAdapterCompatibility,
};

pub struct SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S>,
    A::Compatibility: SystemAdapterCompatibility<S>,
{
    system: SystemSelection<S>,
    adapter: AdapterProviderModule<A>,
    selection: Option<ContractProviderSelection>,
    bridge_mode: AdapterBridgeMode,
}

impl<S, A> Clone for SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S>,
    A::Compatibility: SystemAdapterCompatibility<S>,
{
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            adapter: self.adapter.clone(),
            selection: self.selection.clone(),
            bridge_mode: self.bridge_mode,
        }
    }
}

impl<S, A> SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S>,
    A::Compatibility: SystemAdapterCompatibility<S>,
{
    pub(crate) fn new(
        system: SystemSelection<S>,
        adapter: AdapterProviderModule<A>,
        selection: Option<ContractProviderSelection>,
        bridge_mode: AdapterBridgeMode,
    ) -> Self {
        Self {
            system,
            adapter,
            selection,
            bridge_mode,
        }
    }

    pub fn system(&self) -> &SystemSelection<S> {
        &self.system
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
        SystemSelection<S>,
        AdapterProviderModule<A>,
        ContractProviderSelection,
    ) {
        (
            self.system,
            self.adapter,
            self.selection
                .expect("into_raw_parts is the legacy explicit-realization boundary"),
        )
    }

    pub(crate) fn into_bridge_parts(
        self,
    ) -> (
        SystemSelection<S>,
        AdapterProviderModule<A>,
        Option<ContractProviderSelection>,
        AdapterBridgeMode,
    ) {
        (self.system, self.adapter, self.selection, self.bridge_mode)
    }
}
