mod contract;
mod definition;
mod runtime;

pub use contract::{
    SystemBackedResourceContract, SystemBackedResourceService, system_backed_resource_contract_id,
    system_backed_resource_contract_key,
};
pub use definition::{SystemBackedResource, SystemBackedResourceConfig, system_backed_resource_id};
