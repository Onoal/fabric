//! The normal Fabric 0.1 authoring surface.
//!
//! Advanced Core, Component runtime, Block, and Module machinery remains
//! available through its explicit named SDK modules rather than this prelude.

pub use crate::SdkAuthoringError;
pub use crate::authoring::{
    AdaptableComponentDefinition, AdaptableResourceDefinition, AdaptableSystemDefinition,
    AdapterDefinition, AdapterProviderModule, BuiltFabric, ComponentDefinition,
    ComponentRealization, ComponentResourceBindingManifestEntry, ComponentResourceRequirement,
    ComponentResourceScope, ComponentSpec, ComponentSystemBindingManifestEntry,
    ComponentSystemScope, ContractDependency, Fabric, FabricBuildError, FabricComponents,
    FabricInstance, FabricManifest, IntoFabricResource, IntoFabricSystem, IntoResourceName,
    PrimaryResourceContract, PrimarySystemContract, Requires, ResourceDefinition,
    ResourceManifestEntry, ResourceRealization, ResourceSelection,
    SelfRealizingComponentDefinition, SystemDefinition, SystemManifestEntry, SystemRealization,
    SystemRequires, SystemSelection,
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
