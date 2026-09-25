mod binding;
mod context;
mod instance;
mod requirement;
mod resource;
mod schema;

pub use binding::{ResourceContextBindingProvenance, resource_context_binding_provenance};
pub use context::{ResourceBoundaryId, ResourceContext, ResourceName, ResourceScope};
pub use instance::ResourceInstanceId;
pub use requirement::{
    AdapterResourceSchemaSupport, ResourceCompatibilityError, ResourceCompatibilityRole,
    ResourceRequirement,
};
pub use resource::ResourceId;
pub use schema::{
    ResourceSchemaCompatibilityRequirement, ResourceSchemaDescriptor, ResourceSchemaIdentity,
    ResourceSchemaRequirement, ResourceSchemaVersion,
};
