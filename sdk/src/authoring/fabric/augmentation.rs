use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration};

use super::manifest::ResourceAugmentationManifestEntry;
use super::sealed::Sealed;
use crate::authoring::definitions::{
    PrimaryResourceContract, ResourceAugmentation, ResourceAugmentationDefinition,
};

/// A sealed normal-authoring contribution created by `ResourceAugmentation`.
pub trait IntoFabricResourceAugmentation: Sealed {
    #[doc(hidden)]
    fn into_fabric_resource_augmentation(self) -> FabricResourceAugmentationContribution;
}

pub struct FabricResourceAugmentationContribution {
    entry: ResourceAugmentationManifestEntry,
    module: Box<dyn Module>,
    declaration: ModuleDeclaration,
    selection: ContractProviderSelection,
}

impl FabricResourceAugmentationContribution {
    pub(crate) fn entry(&self) -> &ResourceAugmentationManifestEntry {
        &self.entry
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Box<dyn Module>,
        ModuleDeclaration,
        ContractProviderSelection,
    ) {
        (self.module, self.declaration, self.selection)
    }
}

impl<R, X> Sealed for ResourceAugmentation<R, X>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
}

impl<R, X> IntoFabricResourceAugmentation for ResourceAugmentation<R, X>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
    fn into_fabric_resource_augmentation(self) -> FabricResourceAugmentationContribution {
        let entry = ResourceAugmentationManifestEntry::new(
            X::contract_key().id().clone(),
            X::contract_key().identity().clone(),
            self.resource_id().clone(),
            self.resource_name().clone(),
        );
        let declaration = self.declaration();
        let selection = ContractProviderSelection::new(
            self.module_id().clone(),
            R::primary_contract_key().id().clone(),
            self.target_module_id().clone(),
        );
        FabricResourceAugmentationContribution {
            entry,
            module: Box::new(self),
            declaration,
            selection,
        }
    }
}
