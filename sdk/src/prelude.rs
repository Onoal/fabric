//! The normal Fabric 0.1 authoring surface.
//!
//! Advanced Core, Component runtime, Block, and Module machinery remains
//! available through its explicit named SDK modules rather than this prelude.

pub use crate::SdkAuthoringError;
pub use crate::authoring::{
    AdaptableComponentDefinition, AdaptableResourceDefinition, AdaptableSystemDefinition,
    AdapterDefinition, AdapterProviderModule, BuiltFabric, ComponentAugmentation,
    ComponentAugmentationDefinition, ComponentAugmentationRealization,
    ComponentAugmentationRequirement, ComponentAugmentationSet,
    ComponentAugmentationSetAdapterRealization, ComponentAugmentationSetAttachment,
    ComponentAugmentationSupportDefinition, ComponentDefinition, ComponentRealization,
    ComponentRealizationContract, ComponentResourceBindingManifestEntry,
    ComponentResourceRequirement, ComponentResourceScope, ComponentSpec,
    ComponentSystemBindingManifestEntry, ComponentSystemScope, ContractDependency, Fabric,
    FabricBuildError, FabricComponents, FabricInstance, FabricManifest, IntoFabricResource,
    IntoFabricResourceAugmentation, IntoFabricSystem, IntoFabricSystemAugmentation,
    IntoResourceName, PrimaryResourceContract, PrimarySystemContract, Requires,
    ResourceAugmentation, ResourceAugmentationDefinition, ResourceAugmentationError,
    ResourceAugmentationManifestEntry, ResourceAugmentationRealization,
    ResourceAugmentationRequirement, ResourceAugmentationSupportDefinition, ResourceDefinition,
    ResourceManifestEntry, ResourceRealization, ResourceSelection,
    SelfRealizingComponentDefinition, SystemAugmentation, SystemAugmentationDefinition,
    SystemAugmentationError, SystemAugmentationManifestEntry, SystemAugmentationRealization,
    SystemAugmentationRequirement, SystemAugmentationSupportDefinition, SystemDefinition,
    SystemManifestEntry, SystemRealization, SystemRequires, SystemSelection,
};
pub use crate::component::{
    ComponentError, ComponentId, ComponentRequirementKind, ComponentResourceRequirementName,
    ComponentStatus, InvocationContext, InvocationOrigin, OperationId, OperationKey,
    OperationTypeId, ParticipationState,
};
pub use crate::core::{
    CompositionError, ContractVersion, ContractVersionRequirement, Health, InstanceError,
    InstanceGeneration, InstanceId, LifecycleState,
};
pub use crate::host::{
    HostArchitecture, HostCompatibilityError, HostDescriptor, HostFacilityId, HostOperatingSystem,
    HostRequirement,
};
pub use crate::resource::{
    AdapterResourceSchemaSupport, ResourceCompatibilityError, ResourceId, ResourceName,
    ResourceSchemaDescriptor, ResourceSchemaVersion,
};
pub use crate::system::{
    AdapterSystemSchemaSupport, SystemCompatibilityError, SystemId, SystemSchemaDescriptor,
    SystemSchemaVersion,
};
pub use crate::versions::{
    IntoContractVersion, IntoContractVersionRequirement, contract_requirement, contract_version,
};
pub use crate::{adapter, component, resource, system};
