#![forbid(unsafe_code)]

mod error;
mod model;
#[cfg(test)]
mod source_guards;
#[cfg(test)]
mod tests;

pub use error::ResourceError;
pub use model::{
    AdapterResourceSchemaSupport, ResourceBoundaryId, ResourceCompatibilityError,
    ResourceCompatibilityRole, ResourceContext, ResourceContextBindingProvenance, ResourceId,
    ResourceInstanceId, ResourceName, ResourceRequirement, ResourceSchemaCompatibilityRequirement,
    ResourceSchemaDescriptor, ResourceSchemaIdentity, ResourceSchemaRequirement,
    ResourceSchemaVersion, ResourceScope, resource_context_binding_provenance,
};
