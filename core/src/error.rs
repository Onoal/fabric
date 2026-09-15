use fabric_host::HostCompatibilityError;

use crate::identifiers::{BlockId, CompositionId, ContractId, ModuleId};
use crate::{ContractCompatibilityRequirement, ContractIdentity};

#[derive(Debug, thiserror::Error)]
pub enum CompositionError {
    #[error("composition export `{export_id}` is duplicated")]
    DuplicateCompositionExport { export_id: ContractId },

    #[error("composition export `{export_id}` has no provider for contract `{contract_id}`")]
    MissingExportProvider {
        export_id: ContractId,
        contract_id: ContractId,
    },

    #[error(
        "composition export `{export_id}` has ambiguous providers for contract `{contract_id}`: {providers:?}"
    )]
    AmbiguousExportProvider {
        export_id: ContractId,
        contract_id: ContractId,
        providers: Vec<ModuleId>,
    },

    #[error(
        "composition export `{export_id}` has no compatible provider for contract `{contract_id}`"
    )]
    IncompatibleExportProvider {
        export_id: ContractId,
        contract_id: ContractId,
    },
    #[error("{kind} `{value}` is invalid")]
    InvalidIdentifier { kind: &'static str, value: String },

    #[error("duplicate module id `{module_id}`")]
    DuplicateModuleId { module_id: ModuleId },

    #[error("duplicate block id `{block_id}`")]
    DuplicateBlockId { block_id: BlockId },

    #[error("composition requires an explicit host descriptor for modules {module_ids:?}")]
    HostDescriptorRequired { module_ids: Vec<ModuleId> },

    #[error("module `{module_id}` is incompatible with the supplied host: {source}")]
    HostIncompatible {
        module_id: ModuleId,
        #[source]
        source: HostCompatibilityError,
    },

    #[error("module `{module_id}` has no runtime materializer")]
    MissingRuntimeMaterializer { module_id: ModuleId },

    #[error("module declaration `{declared_module_id}` materialized runtime `{runtime_module_id}`")]
    RuntimeModuleIdMismatch {
        declared_module_id: ModuleId,
        runtime_module_id: ModuleId,
    },

    #[error("module `{module_id}` requires missing contract `{contract_id}`")]
    MissingProvider {
        module_id: ModuleId,
        contract_id: ContractId,
    },

    #[error(
        "module `{module_id}` requires contract `{contract_id}` with compatibility `{required_compatibility}` but only incompatible providers were available"
    )]
    IncompatibleProvider {
        module_id: ModuleId,
        contract_id: ContractId,
        required_compatibility: ContractCompatibilityRequirement,
        available_identities: Vec<ContractIdentity>,
    },

    #[error(
        "module `{module_id}` requires ambiguous contract `{contract_id}` from providers {providers:?}"
    )]
    AmbiguousProvider {
        module_id: ModuleId,
        contract_id: ContractId,
        providers: Vec<ModuleId>,
    },

    #[error(
        "composition selected duplicate providers for consumer `{consumer}` contract `{contract_id}`: `{first_provider}` and `{second_provider}`"
    )]
    DuplicateContractProviderSelection {
        consumer: ModuleId,
        contract_id: ContractId,
        first_provider: ModuleId,
        second_provider: ModuleId,
    },

    #[error(
        "composition selected unknown consumer `{consumer}` for contract `{contract_id}` provider `{provider}`"
    )]
    UnknownSelectionConsumer {
        consumer: ModuleId,
        contract_id: ContractId,
        provider: ModuleId,
    },

    #[error(
        "composition selected unknown provider `{provider}` for consumer `{consumer}` contract `{contract_id}`"
    )]
    UnknownSelectionProvider {
        consumer: ModuleId,
        contract_id: ContractId,
        provider: ModuleId,
    },

    #[error(
        "composition selected provider `{provider}` for undeclared consumer requirement `{contract_id}` on module `{consumer}`"
    )]
    SelectedUndeclaredRequirement {
        consumer: ModuleId,
        contract_id: ContractId,
        provider: ModuleId,
    },

    #[error(
        "composition selected provider `{provider}` for consumer `{consumer}` contract `{contract_id}` but that provider does not declare the contract"
    )]
    SelectedProviderMissingContract {
        consumer: ModuleId,
        contract_id: ContractId,
        provider: ModuleId,
    },

    #[error(
        "composition selected provider `{provider}` for consumer `{consumer}` contract `{contract_id}` with compatibility `{required_compatibility}` but the selected provider exports only incompatible identities"
    )]
    SelectedProviderIncompatible {
        consumer: ModuleId,
        contract_id: ContractId,
        provider: ModuleId,
        required_compatibility: ContractCompatibilityRequirement,
        available_identities: Vec<ContractIdentity>,
    },

    #[error("module `{module_id}` declares duplicate requirements for contract `{contract_id}`")]
    DuplicateContractRequirement {
        module_id: ModuleId,
        contract_id: ContractId,
    },

    #[error(
        "module `{module_id}` declares duplicate provided contract `{contract_id}` with identity `{identity}`"
    )]
    DuplicateProvidedContractDeclaration {
        module_id: ModuleId,
        contract_id: ContractId,
        identity: ContractIdentity,
    },

    #[error("module `{module_id}` exported undeclared contract `{contract_id}`")]
    UndeclaredProvidedContract {
        module_id: ModuleId,
        contract_id: ContractId,
    },

    #[error("module `{module_id}` did not export declared contract `{contract_id}`")]
    MissingDeclaredExport {
        module_id: ModuleId,
        contract_id: ContractId,
    },

    #[error(
        "module `{module_id}` attempted to resolve undeclared or unbound contract `{contract_id}`"
    )]
    UnboundRequirement {
        module_id: ModuleId,
        contract_id: ContractId,
    },

    #[error(
        "module `{module_id}` bound contract `{contract_id}` with requested compatibility `{requested_compatibility}` but declared `{declared_compatibility}`"
    )]
    RequirementDeclarationMismatch {
        module_id: ModuleId,
        contract_id: ContractId,
        declared_compatibility: ContractCompatibilityRequirement,
        requested_compatibility: ContractCompatibilityRequirement,
    },

    #[error("module `{module_id}` received contract `{contract_id}` with the wrong Rust type")]
    ContractTypeMismatch {
        module_id: ModuleId,
        contract_id: ContractId,
    },

    #[error("composition contains a dependency cycle involving modules {module_ids:?}")]
    DependencyCycle { module_ids: Vec<ModuleId> },

    #[error(
        "composition `{composition_id}` failed during `{phase}` for module `{module_id}`: {source}"
    )]
    ModuleFailure {
        composition_id: CompositionId,
        module_id: ModuleId,
        phase: &'static str,
        #[source]
        source: ModuleError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("instance `{instance_id}` is not in a state that allows `{action}`")]
    InvalidLifecycleTransition {
        instance_id: crate::identifiers::InstanceId,
        action: &'static str,
    },

    #[error("module `{module_id}` failed during `{phase}`: {source}")]
    ModuleFailure {
        module_id: ModuleId,
        phase: &'static str,
        #[source]
        source: ModuleError,
    },
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ModuleError {
    message: String,
}

impl ModuleError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
