pub use fabric_resource::{
    AdapterResourceSchemaSupport, ResourceBoundaryId, ResourceCompatibilityError,
    ResourceCompatibilityRole, ResourceContext, ResourceContextBindingProvenance, ResourceError,
    ResourceId, ResourceInstanceId, ResourceName, ResourceRequirement,
    ResourceSchemaCompatibilityRequirement, ResourceSchemaDescriptor, ResourceSchemaIdentity,
    ResourceSchemaRequirement, ResourceSchemaVersion, ResourceScope,
    resource_context_binding_provenance,
};
use std::future::Future;
use std::pin::Pin;

/// Awaitable execution boundary for Fabric Resource operations.
///
/// Resource operations are uniformly awaitable so immediate local
/// realizations and event-loop-backed realizations share one capability
/// contract without requiring a runtime, executor, or globally `Send` future.
pub type ResourceFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
