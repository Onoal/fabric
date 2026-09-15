#![forbid(unsafe_code)]
//! Experimental, extraction-era Resource Registry machinery.
//!
//! This crate is retained outside Fabric's canonical 0.1 Resource path. Its
//! `ResourceId`-keyed descriptors are local registry state, not Fabric
//! occurrence identity, selection, realization, or inspection authority.
//! Canonical Resource occurrences are `ResourceId + ResourceName` and belong
//! to the normal Resource and Composition machinery.

mod contract;
mod error;
mod model;
mod native;

#[cfg(test)]
mod source_guards;
#[cfg(test)]
mod tests;

pub use contract::{
    ResourceRegistry, ResourceRegistryService, resource_registry_contract_id,
    resource_registry_contract_key,
};
pub use error::ResourceRegistryError;
pub use model::{
    ResourceConfiguration, ResourceConfigurationFacet, ResourceConfigurationKind,
    ResourceDescriptor, ResourceFacetId, ResourceInspection, ResourceInspectionEntry,
    ResourceInspectionFacet, ResourceInspectionValue, ResourceInterface, ResourceInterfaceRole,
    resource_configuration_facet_id, resource_inspection_facet_id,
};
pub use native::ResourceRegistryModule;
