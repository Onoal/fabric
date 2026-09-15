use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration};
use fabric_resource::AdapterResourceSchemaSupport;

use super::manifest::ResourceManifestEntry;
use super::sealed::Sealed;
use crate::authoring::definitions::{
    AdaptableResourceDefinition, AdapterDefinition, ResourceDefinition, ResourceRealization,
    ResourceSelection,
};

pub trait IntoFabricResource: Sealed {
    #[doc(hidden)]
    fn into_fabric_resource(self) -> FabricResourceContribution;
}

pub struct FabricResourceContribution {
    entry: ResourceManifestEntry,
    modules: Vec<Box<dyn Module>>,
    declarations: Vec<ModuleDeclaration>,
    provider_selections: Vec<ContractProviderSelection>,
}

impl FabricResourceContribution {
    pub(crate) fn entry(&self) -> &ResourceManifestEntry {
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
        entry: ResourceManifestEntry,
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

impl<R> Sealed for ResourceSelection<R> where R: ResourceDefinition {}

impl<R> IntoFabricResource for ResourceSelection<R>
where
    R: ResourceDefinition,
{
    fn into_fabric_resource(self) -> FabricResourceContribution {
        let entry = ResourceManifestEntry::new(R::resource_id(), self.name().clone(), R::schema());
        let declarations = vec![self.declaration()];
        FabricResourceContribution::new(entry, vec![Box::new(self)], declarations, Vec::new())
    }
}

impl<R, A> Sealed for ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R, Compatibility = AdapterResourceSchemaSupport>,
{
}

impl<R, A> IntoFabricResource for ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R, Compatibility = AdapterResourceSchemaSupport>,
{
    fn into_fabric_resource(self) -> FabricResourceContribution {
        let (resource, adapter, selection) = self.into_raw_parts();
        let entry =
            ResourceManifestEntry::new(R::resource_id(), resource.name().clone(), R::schema());
        let declarations = vec![resource.declaration(), adapter.declaration()];
        FabricResourceContribution::new(
            entry,
            vec![Box::new(resource), Box::new(adapter)],
            declarations,
            vec![selection],
        )
    }
}
