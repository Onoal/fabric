#![forbid(unsafe_code)]

pub mod adapter;
pub mod authoring;
pub mod component;
pub mod contracts;
pub mod core;
pub mod experimental;
pub mod host;
pub mod ids;
pub mod prelude;
pub mod resource;
pub mod system;
pub mod versions;

mod error;

#[cfg(test)]
mod source_guards;
#[cfg(test)]
mod tests;

pub use fabric_sdk_macros::{adapter, component, resource, system};

pub use authoring::{
    AdaptableComponentDefinition, AdaptableResourceDefinition, AdaptableSystemDefinition,
    AdapterDefinition, AdapterProviderModule, BuiltFabric, ComponentAugmentation,
    ComponentAugmentationDefinition, ComponentAugmentationRealization,
    ComponentAugmentationRequirement, ComponentAugmentationSet,
    ComponentAugmentationSetAdapterRealization, ComponentAugmentationSetAttachment,
    ComponentAugmentationSetRealization, ComponentAugmentationSupportDefinition,
    ComponentDefinition, ComponentRealization, ComponentRealizationContract,
    ComponentResourceBindingManifestEntry, ComponentResourceRequirement, ComponentResourceScope,
    ComponentSpec, ComponentSystemBindingManifestEntry, ComponentSystemScope, ContractDependency,
    Fabric, FabricBuildError, FabricComponents, FabricInstance, FabricManifest, IntoFabricResource,
    IntoFabricResourceAugmentation, IntoFabricSystem, IntoFabricSystemAugmentation,
    IntoResourceName, PrimaryResourceContract, PrimarySystemContract, Requires,
    ResourceAugmentation, ResourceAugmentationDefinition, ResourceAugmentationError,
    ResourceAugmentationManifestEntry, ResourceAugmentationRealization,
    ResourceAugmentationRequirement, ResourceAugmentationSupportDefinition, ResourceDefinition,
    ResourceManifestEntry, ResourceRealization, ResourceSelection, RuntimeContext, RuntimeState,
    SelfRealizingComponentDefinition, StatefulAdapterDefinition, StatefulRuntimeAuthoring,
    SystemAugmentation, SystemAugmentationDefinition, SystemAugmentationError,
    SystemAugmentationManifestEntry, SystemAugmentationRealization, SystemAugmentationRequirement,
    SystemAugmentationSupportDefinition, SystemDefinition, SystemManifestEntry, SystemRealization,
    SystemRequires, SystemSelection,
};
pub use component::{
    ComponentAugmentationRuntimePreparation, ComponentError, ComponentId, ComponentRequirementKind,
    ComponentResourceRequirementName, ComponentRuntimeContribution, ComponentRuntimePreparation,
    ComponentRuntimeTeardownError, ComponentRuntimeTeardownFailure, ComponentStatus,
    InvocationContext, InvocationOrigin, OperationId, OperationKey, OperationTypeId,
    ParticipationState,
};
pub use core::{
    CompositionError, CompositionId, ContractId, ContractIdentity, ContractKey, ContractVersion,
    ContractVersionRequirement, Health, InstanceError, InstanceGeneration, InstanceId,
    LifecycleState, ModuleError, RuntimeCleanupError,
};
pub use error::SdkAuthoringError;
pub use host::{
    HostArchitecture, HostCompatibilityError, HostDescriptor, HostFacilityId, HostOperatingSystem,
    HostRequirement,
};
pub use resource::{
    AdapterResourceSchemaSupport, ResourceCompatibilityError, ResourceId, ResourceName,
    ResourceSchemaDescriptor, ResourceSchemaVersion,
};
pub use system::{
    AdapterSystemSchemaSupport, SystemCompatibilityError, SystemId, SystemSchemaDescriptor,
    SystemSchemaVersion,
};
pub use versions::{
    IntoContractVersion, IntoContractVersionRequirement, contract_requirement, contract_version,
};
