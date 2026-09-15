use fabric_core::ContractProviderSelection;
use fabric_system::AdapterSystemSchemaSupport;

use super::{AdaptableSystemDefinition, SystemSelection};
use crate::authoring::definitions::{AdapterDefinition, AdapterProviderModule};

pub struct SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S, SchemaSupport = AdapterSystemSchemaSupport>,
{
    system: SystemSelection<S>,
    adapter: AdapterProviderModule<A>,
    selection: ContractProviderSelection,
}

impl<S, A> Clone for SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S, SchemaSupport = AdapterSystemSchemaSupport>,
{
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
            adapter: self.adapter.clone(),
            selection: self.selection.clone(),
        }
    }
}

impl<S, A> SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S, SchemaSupport = AdapterSystemSchemaSupport>,
{
    pub(crate) fn new(
        system: SystemSelection<S>,
        adapter: AdapterProviderModule<A>,
        selection: ContractProviderSelection,
    ) -> Self {
        Self {
            system,
            adapter,
            selection,
        }
    }

    pub fn system(&self) -> &SystemSelection<S> {
        &self.system
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
        SystemSelection<S>,
        AdapterProviderModule<A>,
        ContractProviderSelection,
    ) {
        (self.system, self.adapter, self.selection)
    }
}
