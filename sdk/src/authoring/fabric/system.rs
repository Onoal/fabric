use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration};
use fabric_system::AdapterSystemSchemaSupport;

use super::manifest::SystemManifestEntry;
use super::sealed::Sealed;
use crate::authoring::definitions::AdapterDefinition;
use crate::authoring::system::{
    AdaptableSystemDefinition, SystemDefinition, SystemRealization, SystemSelection,
};

pub trait IntoFabricSystem: Sealed {
    #[doc(hidden)]
    fn into_fabric_system(self) -> FabricSystemContribution;
}

pub struct FabricSystemContribution {
    entry: SystemManifestEntry,
    modules: Vec<Box<dyn Module>>,
    declarations: Vec<ModuleDeclaration>,
    provider_selections: Vec<ContractProviderSelection>,
}

impl FabricSystemContribution {
    pub(crate) fn entry(&self) -> &SystemManifestEntry {
        &self.entry
    }

    pub(crate) fn modules(self) -> Vec<Box<dyn Module>> {
        self.modules
    }

    pub(crate) fn declarations(&self) -> &[ModuleDeclaration] {
        &self.declarations
    }

    pub(crate) fn provider_selections(&self) -> &[ContractProviderSelection] {
        &self.provider_selections
    }

    fn new(
        entry: SystemManifestEntry,
        modules: Vec<Box<dyn Module>>,
        declarations: Vec<ModuleDeclaration>,
        provider_selections: Vec<ContractProviderSelection>,
    ) -> Self {
        Self {
            entry,
            modules,
            declarations,
            provider_selections,
        }
    }
}

impl<S> Sealed for SystemSelection<S> where S: SystemDefinition {}

impl<S> IntoFabricSystem for SystemSelection<S>
where
    S: SystemDefinition,
{
    fn into_fabric_system(self) -> FabricSystemContribution {
        let entry = SystemManifestEntry::new(S::system_id(), S::schema());
        let declarations = vec![self.declaration()];
        FabricSystemContribution::new(entry, vec![Box::new(self)], declarations, Vec::new())
    }
}

impl<S, A> Sealed for SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S, Compatibility = AdapterSystemSchemaSupport>,
{
}

impl<S, A> IntoFabricSystem for SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S, Compatibility = AdapterSystemSchemaSupport>,
{
    fn into_fabric_system(self) -> FabricSystemContribution {
        let (system, adapter, selection) = self.into_raw_parts();
        let entry = SystemManifestEntry::new(S::system_id(), S::schema());
        let declarations = vec![system.declaration(), adapter.declaration()];
        FabricSystemContribution::new(
            entry,
            vec![Box::new(system), Box::new(adapter)],
            declarations,
            vec![selection],
        )
    }
}
