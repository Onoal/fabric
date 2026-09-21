//! Optional convenience imports for Fabric's normal high-level authoring API.
//!
//! `use fabric::*` is the canonical root-first style. This prelude mirrors
//! that normal surface for existing users; it neither contains raw Core rails
//! nor [`crate::experimental`] APIs.

pub use crate::{
    AdaptableComponentDefinition, AdaptableResourceDefinition, AdaptableSystemDefinition,
    AdapterDefinition, AdapterProviderModule, AdapterResourceSchemaSupport,
    AdapterSystemSchemaSupport, BuiltFabric, ComponentAugmentation,
    ComponentAugmentationDefinition, ComponentAugmentationRealization,
    ComponentAugmentationRequirement, ComponentAugmentationSet,
    ComponentAugmentationSetAdapterRealization, ComponentAugmentationSetAttachment,
    ComponentAugmentationSetRealization, ComponentAugmentationSupportDefinition,
    ComponentDefinition, ComponentError, ComponentId, ComponentRealization,
    ComponentRealizationContract, ComponentRequirementKind, ComponentResourceBindingManifestEntry,
    ComponentResourceRequirement, ComponentResourceRequirementName, ComponentResourceScope,
    ComponentSpec, ComponentStatus, ComponentSystemBindingManifestEntry, ComponentSystemScope,
    CompositionError, CompositionId, ContractDependency, ContractId, ContractIdentity, ContractKey,
    ContractVersion, ContractVersionRequirement, Fabric, FabricBuildError, FabricComponents,
    FabricInstance, FabricManifest, Health, HostArchitecture, HostCompatibilityError,
    HostDescriptor, HostFacilityId, HostOperatingSystem, HostRequirement, InstanceError,
    InstanceGeneration, InstanceId, IntoContractVersion, IntoContractVersionRequirement,
    IntoFabricResource, IntoFabricResourceAugmentation, IntoFabricSystem,
    IntoFabricSystemAugmentation, IntoResourceName, InvocationContext, InvocationOrigin,
    LifecycleState, ModuleError, OperationId, OperationKey, OperationTypeId, ParticipationState,
    PrimaryResourceContract, PrimarySystemContract, Requires, ResourceAugmentation,
    ResourceAugmentationDefinition, ResourceAugmentationError, ResourceAugmentationManifestEntry,
    ResourceAugmentationRealization, ResourceAugmentationRequirement,
    ResourceAugmentationSupportDefinition, ResourceCompatibilityError, ResourceDefinition,
    ResourceId, ResourceManifestEntry, ResourceName, ResourceRealization, ResourceSchemaDescriptor,
    ResourceSchemaVersion, ResourceSelection, RuntimeCleanupError, RuntimeContext, RuntimeState,
    SdkAuthoringError, SelfRealizingComponentDefinition, StatefulAdapterDefinition,
    StatefulRuntimeAuthoring, SystemAugmentation, SystemAugmentationDefinition,
    SystemAugmentationError, SystemAugmentationManifestEntry, SystemAugmentationRealization,
    SystemAugmentationRequirement, SystemAugmentationSupportDefinition, SystemCompatibilityError,
    SystemDefinition, SystemId, SystemManifestEntry, SystemRealization, SystemRequires,
    SystemSchemaDescriptor, SystemSchemaVersion, SystemSelection, adapter, component,
    contract_requirement, contract_version, resource, system,
};
