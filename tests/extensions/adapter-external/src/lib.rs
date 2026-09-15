mod resource_adapter;
mod system_adapter;

#[cfg(test)]
mod tests;

pub use resource_adapter::{
    ExternalCounterAdapter, ExternalCounterAdapterConfig, HostBoundExternalCounterAdapter,
    HostBoundExternalCounterAdapterConfig, external_counter_host_facility,
};
pub use system_adapter::{
    ExternalOperationsAdapter, ExternalOperationsAdapterConfig,
    ExternalOperationsWithSystemAdapter, ExternalOperationsWithSystemAdapterConfig,
};
