use crate::{SystemId, SystemSchemaCompatibilityRequirement, SystemSchemaIdentity};

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum SystemCompatibilityError {
    #[error(
        "system `{system}` uses an explicit realization contract; canonical Adapter authoring applies only when the System API is its realization contract"
    )]
    CanonicalAdapterRequiresApiRealization { system: SystemId },
    #[error("system input is invalid: {message}")]
    InvalidInput { message: String },
    #[error("system identity mismatch: expected `{expected}`, got `{actual}`")]
    SystemIdentityMismatch {
        expected: SystemId,
        actual: SystemId,
    },
    #[error("system schema incompatibility: required `{required}`, got `{actual}`")]
    SchemaIncompatible {
        required: SystemSchemaCompatibilityRequirement,
        actual: SystemSchemaIdentity,
    },
}
