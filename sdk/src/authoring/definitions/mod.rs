mod adapter_definition;
mod adapter_provider_module;
mod component_augmentation;
mod component_definition;
mod contract_dependency;
mod primary_resource_contract;
mod relation_target;
mod requires;
mod resource_augmentation;
mod resource_definition;
mod resource_realization;
mod resource_selection;

pub use adapter_definition::{
    AdapterBridgeMode, AdapterDefinition, CanonicalAdapterSupport, ResourceAdapterCompatibility,
    SystemAdapterCompatibility,
};
pub use adapter_provider_module::AdapterProviderModule;
pub use component_augmentation::{
    ComponentAugmentation, ComponentAugmentationDefinition, ComponentAugmentationRealization,
    ComponentAugmentationRequirement, ComponentAugmentationSet,
    ComponentAugmentationSetAdapterRealization, ComponentAugmentationSetAttachment,
    ComponentAugmentationSetRealization, ComponentAugmentationSupportDefinition,
    ComponentAugmentedAdapterRealization,
};
pub(crate) use component_definition::ComponentSpecParts;
pub use component_definition::{
    AdaptableComponentDefinition, ComponentDefinition, ComponentRealization,
    ComponentRealizationContract, ComponentRelationCompatibility, ComponentRelationTarget,
    ComponentResourceRequirement, ComponentResourceScope, ComponentSpec, ComponentSystemScope,
    SelfRealizingComponentDefinition,
};
pub use contract_dependency::ContractDependency;
pub use primary_resource_contract::PrimaryResourceContract;
pub use relation_target::RelationTarget;
pub use requires::Requires;
pub use resource_augmentation::{
    ResourceAugmentation, ResourceAugmentationDefinition, ResourceAugmentationError,
    ResourceAugmentationRealization, ResourceAugmentationRequirement,
    ResourceAugmentationSupportDefinition,
};
pub use resource_definition::{AdaptableResourceDefinition, IntoResourceName, ResourceDefinition};
pub use resource_realization::ResourceRealization;
pub use resource_selection::ResourceSelection;
