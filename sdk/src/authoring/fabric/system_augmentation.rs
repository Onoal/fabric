use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration};

use super::manifest::SystemAugmentationManifestEntry;
use super::sealed::Sealed;
use crate::authoring::{
    PrimarySystemContract, SystemAugmentation, SystemAugmentationDefinition,
    SystemAugmentationRealization, SystemAugmentationSupportDefinition,
};

/// A sealed normal-authoring contribution created by `SystemAugmentation`.
pub trait IntoFabricSystemAugmentation: Sealed {
    #[doc(hidden)]
    fn into_fabric_system_augmentation(self) -> FabricSystemAugmentationContribution;
}

pub struct FabricSystemAugmentationContribution {
    entry: SystemAugmentationManifestEntry,
    modules: Vec<Box<dyn Module>>,
    declarations: Vec<ModuleDeclaration>,
    selections: Vec<ContractProviderSelection>,
}

impl FabricSystemAugmentationContribution {
    pub(crate) fn entry(&self) -> &SystemAugmentationManifestEntry {
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

impl<S, X> Sealed for SystemAugmentation<S, X>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
{
}

impl<S, X> IntoFabricSystemAugmentation for SystemAugmentation<S, X>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
{
    fn into_fabric_system_augmentation(self) -> FabricSystemAugmentationContribution {
        let entry = SystemAugmentationManifestEntry::new(
            X::contract_key().id().clone(),
            X::contract_key().identity().clone(),
            self.system_id().clone(),
            None,
        );
        let declaration = self.declaration();
        let selection = ContractProviderSelection::new(
            self.module_id().clone(),
            S::primary_contract_key().id().clone(),
            self.target_module_id().clone(),
        );
        FabricSystemAugmentationContribution {
            entry,
            modules: vec![Box::new(self)],
            declarations: vec![declaration],
            selections: vec![selection],
        }
    }
}

impl<S, X, P> Sealed for SystemAugmentationRealization<S, X, P>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
    P: SystemAugmentationSupportDefinition<S, X>,
{
}

impl<S, X, P> IntoFabricSystemAugmentation for SystemAugmentationRealization<S, X, P>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
    P: SystemAugmentationSupportDefinition<S, X>,
{
    fn into_fabric_system_augmentation(self) -> FabricSystemAugmentationContribution {
        let attachment = self.attachment().clone();
        let entry = SystemAugmentationManifestEntry::new(
            X::contract_key().id().clone(),
            X::contract_key().identity().clone(),
            attachment.system_id().clone(),
            Some(self.provider_module_id().clone()),
        );
        let attachment_declaration = attachment.declaration();
        let support_declaration = self.declaration();
        let attachment_selection = ContractProviderSelection::new(
            attachment.module_id().clone(),
            S::primary_contract_key().id().clone(),
            attachment.target_module_id().clone(),
        );
        let support_selection = ContractProviderSelection::new(
            self.provider_module_id().clone(),
            S::primary_contract_key().id().clone(),
            attachment.target_module_id().clone(),
        );
        FabricSystemAugmentationContribution {
            entry,
            modules: vec![Box::new(attachment), Box::new(self)],
            declarations: vec![attachment_declaration, support_declaration],
            selections: vec![attachment_selection, support_selection],
        }
    }
}
