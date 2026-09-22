use super::manifest::SystemManifestEntry;
use super::sealed::Sealed;
use crate::authoring::definitions::{
    AdapterBridgeMode, AdapterDefinition, SystemAdapterCompatibility,
};
use crate::authoring::system::{
    AdaptableSystemDefinition, SystemDefinition, SystemRealization, SystemSelection,
};
use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration};

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
    A: AdapterDefinition<Target = S>,
    A::Compatibility: SystemAdapterCompatibility<S>,
{
}

impl<S, A> IntoFabricSystem for SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S>,
    A::Compatibility: SystemAdapterCompatibility<S>,
{
    fn into_fabric_system(self) -> FabricSystemContribution {
        let (system, adapter, selection, bridge_mode) = self.into_bridge_parts();
        let entry = SystemManifestEntry::new(S::system_id(), S::schema());
        if bridge_mode == AdapterBridgeMode::SemanticApi {
            // See the Resource equivalent: semantic Relations constrain the
            // Adapter-owned live participant without reviving a System proxy.
            let declaration = adapter.declaration();
            FabricSystemContribution::new(
                entry,
                vec![Box::new(adapter)],
                vec![declaration],
                Vec::new(),
            )
        } else {
            let declarations = vec![system.declaration(), adapter.declaration()];
            FabricSystemContribution::new(
                entry,
                vec![Box::new(system), Box::new(adapter)],
                declarations,
                vec![selection.expect("legacy adapter realization selects its provider")],
            )
        }
    }
}
