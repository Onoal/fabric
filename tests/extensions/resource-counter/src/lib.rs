mod adapters;
mod definition;
mod model;
mod package_adapter;
mod package_only;

#[cfg(test)]
mod tests;

pub use adapters::{
    FixedCounterAdapter, FixedCounterAdapterConfig, IncompatibleSchemaCounterAdapter,
    IncompatibleSchemaCounterAdapterConfig, MissingContractCounterAdapter,
    ThirdPartyStyleCounterAdapter, WrongVersionCounterAdapter, WrongVersionCounterAdapterConfig,
};
pub use definition::AdaptedCounterRealization;
pub use definition::direct_counter;
pub use definition::{AdaptedCounter, AdaptedCounterConfig, DerivedCounter, DerivedCounterConfig};
pub use definition::{DirectCounter, DirectCounterConfig};
pub use model::{CounterValue, counter_resource_id};
pub use package_adapter::{PackageOnlyAdapter, PackageOnlyAdapterConfig};
pub use package_only::{PackageOnlyCapability, PackageOnlyConfig, PackageOnlyResource};
