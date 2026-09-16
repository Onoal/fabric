use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration};

use super::manifest::ResourceAugmentationManifestEntry;
use super::sealed::Sealed;
use crate::authoring::definitions::{
    PrimaryResourceContract, ResourceAugmentation, ResourceAugmentationDefinition,
    ResourceAugmentationRealization, ResourceAugmentationSupportDefinition,
};

/// A sealed normal-authoring contribution created by `ResourceAugmentation`.
pub trait IntoFabricResourceAugmentation: Sealed {
    #[doc(hidden)]
    fn into_fabric_resource_augmentation(self) -> FabricResourceAugmentationContribution;
}

pub struct FabricResourceAugmentationContribution {
    entry: ResourceAugmentationManifestEntry,
    modules: Vec<Box<dyn Module>>,
    declarations: Vec<ModuleDeclaration>,
    selections: Vec<ContractProviderSelection>,
}

impl FabricResourceAugmentationContribution {
    pub(crate) fn entry(&self) -> &ResourceAugmentationManifestEntry {
        &self.entry
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<Box<dyn Module>>,
        Vec<ModuleDeclaration>,
        Vec<ContractProviderSelection>,
    ) {
        (self.modules, self.declarations, self.selections)
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
            modules: vec![Box::new(self)],
            declarations: vec![declaration],
            selections: vec![selection],
        }
    }
}

impl<R, X, S> Sealed for ResourceAugmentationRealization<R, X, S>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
    S: ResourceAugmentationSupportDefinition<R, X>,
{
}

impl<R, X, S> IntoFabricResourceAugmentation for ResourceAugmentationRealization<R, X, S>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
    S: ResourceAugmentationSupportDefinition<R, X>,
{
    fn into_fabric_resource_augmentation(self) -> FabricResourceAugmentationContribution {
        let attachment = self.attachment().clone();
        let entry = ResourceAugmentationManifestEntry::new(
            X::contract_key().id().clone(),
            X::contract_key().identity().clone(),
            attachment.resource_id().clone(),
            attachment.resource_name().clone(),
        );
        let attachment_declaration = attachment.declaration();
        let support_declaration = self.declaration();
        let attachment_selection = ContractProviderSelection::new(
            attachment.module_id().clone(),
            R::primary_contract_key().id().clone(),
            attachment.target_module_id().clone(),
        );
        let support_selection = ContractProviderSelection::new(
            self.provider_module_id().clone(),
            R::primary_contract_key().id().clone(),
            attachment.target_module_id().clone(),
        );
        FabricResourceAugmentationContribution {
            entry,
            modules: vec![Box::new(attachment), Box::new(self)],
            declarations: vec![attachment_declaration, support_declaration],
            selections: vec![attachment_selection, support_selection],
        }
    }
}
