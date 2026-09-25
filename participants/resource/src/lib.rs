//! Resource identity, occurrence, and compatibility primitives for Fabric.
//!
//! A Resource is an occurrence-based semantic capability. This crate carries
//! its lower typed identity and selection model; it does not make a Resource a
//! concrete Adapter realization or live runtime owner. Normal authors should
//! use `fabric::resource!` through the `fabric` umbrella crate.

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
