mod configuration;
mod descriptor;
mod facet;
mod inspection;
mod interface;
mod key;

pub use configuration::{
    ResourceConfiguration, ResourceConfigurationFacet, ResourceConfigurationKind,
};
pub use descriptor::ResourceDescriptor;
pub use facet::{ResourceFacetId, resource_configuration_facet_id, resource_inspection_facet_id};
pub use inspection::{
    ResourceInspection, ResourceInspectionEntry, ResourceInspectionFacet, ResourceInspectionValue,
};
pub use interface::{ResourceInterface, ResourceInterfaceRole};
pub(crate) use key::resource_registry_module_id;
