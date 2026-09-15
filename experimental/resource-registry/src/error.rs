use fabric_core::ModuleId;
use fabric_resource::ResourceId;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ResourceRegistryError {
    #[error("resource registry is unavailable")]
    Unavailable,

    #[error("resource registry input is invalid: {message}")]
    InvalidInput { message: String },

    #[error(
        "resource `{resource_id}` is already participating through module `{existing_module_id}`"
    )]
    DuplicateResourceParticipation {
        resource_id: ResourceId,
        existing_module_id: ModuleId,
    },

    #[error("module `{module_id}` is already participating as resource `{existing_resource_id}`")]
    DuplicateResourceParticipant {
        module_id: ModuleId,
        existing_resource_id: ResourceId,
    },

    #[error("module `{module_id}` is not participating in the resource registry")]
    UnknownResourceParticipant { module_id: ModuleId },

    #[error("resource `{resource_id}` is not participating in the resource registry")]
    UnknownResource { resource_id: ResourceId },

    #[error("resource `{resource_id}` does not participate in registry configuration")]
    ConfigurationUnsupported { resource_id: ResourceId },

    #[error("resource `{resource_id}` does not participate in safe inspection")]
    InspectionUnsupported { resource_id: ResourceId },

    #[error("resource inspection failed: {message}")]
    InspectionFailed { message: String },

    #[error(
        "resource `{resource_id}` claimed interface `{contract_id}` with role `{role}` that does not match core module contract truth"
    )]
    InterfaceClaimMismatch {
        resource_id: ResourceId,
        contract_id: String,
        role: String,
    },

    #[error("resource configuration was rejected: {message}")]
    ConfigurationRejected { message: String },
}

impl ResourceRegistryError {
    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }
}
