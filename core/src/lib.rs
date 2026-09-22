mod block;
mod composition;
mod contract;
mod derived_provider;
mod error;
mod export;
mod health;
mod host_materialization;
mod identifiers;
mod instance;
mod lifecycle;
mod module;
mod module_runtime;

#[cfg(test)]
mod source_guards;
#[cfg(test)]
mod tests;

pub use block::{Block, BlockBuilder, BlockReport, ModuleReport};
pub use composition::{
    Composition, CompositionBuilder, ContractProviderSelection, ModuleBindings, ResolvedContract,
};
pub use contract::{
    ContractCompatibilityRequirement, ContractIdentity, ContractKey, ContractRequirement,
    ContractRequirementDeclaration, ContractVersion, ContractVersionRequirement, ModuleContract,
    ProvidedContractDeclaration,
};
pub use derived_provider::{DerivedContractFactory, DerivedContractProvider};
pub use error::{
    CompositionError, InstanceError, ModuleCleanupFailure, ModuleError, RuntimeCleanupError,
};
pub use export::{CompositionExport, CompositionExportDeclaration};
pub use health::Health;
pub use host_materialization::HostMaterializationRequirement;
pub use identifiers::{BlockId, CompositionId, ContractId, InstanceId, ModuleId};
pub use instance::{Instance, InstanceGeneration, InstanceReport, InstanceRuntimeContext};
pub use lifecycle::LifecycleState;
pub use module::{Module, ModuleDeclaration, ModuleFactory, module_factory};
pub use module_runtime::ModuleRuntime;
