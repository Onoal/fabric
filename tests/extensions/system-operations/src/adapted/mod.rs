mod adapter;
mod definition;
mod host_bound;

#[cfg(test)]
pub(crate) use adapter::LifecycleCaptureOperationsAdapter;
pub use adapter::{
    AlternateOperationsAdapter, AlternateOperationsAdapterConfig, FixedOperationsAdapter,
    FixedOperationsAdapterConfig, IncompatibleSchemaOperationsAdapter,
    IncompatibleSchemaOperationsAdapterConfig, MissingContractOperationsAdapter,
    WrongVersionOperationsAdapter, WrongVersionOperationsAdapterConfig,
};
pub use definition::adapted_operations::raw::{
    ApiContract as AdaptedOperationsContract, ApiService as AdaptedOperationsService,
    primary_contract_id as adapted_operations_contract_id,
    primary_contract_key as adapted_operations_contract_key,
};
pub use definition::adapted_operations::realization::raw::{
    AdapterContract as AdaptedOperationsRealizationContract,
    AdapterService as AdaptedOperationsRealizationService,
    contract_id as adapted_operations_realization_contract_id,
};
pub use definition::{
    AdaptedOperations, AdaptedOperationsConfig, AdaptedOperationsRealization,
    BoundAdaptedOperationsRealization, adapted_operations_contract_version,
    adapted_operations_realization_contract_key, adapted_operations_realization_contract_version,
    adapted_operations_realization_requirement, adapted_operations_schema_version,
    adapted_operations_system_id,
};
pub use host_bound::{
    HostBoundOperationsAdapter, HostBoundOperationsAdapterConfig, host_bound_operations_facility,
};
