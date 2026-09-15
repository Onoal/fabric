//! A package-style Adapter defined entirely through public
//! authoring APIs with no runtime provider materialization.
//!
//! This module intentionally contains:
//!
//! - a target association (`AdaptedCounter`, which owns the realization contract)
//! - target-scoped schema support for the target's schema
//! - declarative provider metadata (the target-owned realization contract)
//! - declarative Host requirement semantics via the inherited default
//!
//! and intentionally omits any runtime provider implementation.
//!
//! A `src/tests.rs` source guard pins this negative space so the package
//! cannot silently regain a runtime obligation.

use fabric_core::{ContractVersion, ModuleDeclaration, ModuleId};
use fabric_resource::AdapterResourceSchemaSupport;
use fabric_sdk::prelude::{AdapterDefinition, ResourceDefinition};

use crate::{AdaptedCounter, definition::adapted_counter};

#[derive(Clone)]
pub struct PackageOnlyAdapterConfig {
    pub label: String,
}

#[derive(Clone)]
pub struct PackageOnlyAdapter {
    config: PackageOnlyAdapterConfig,
}

impl PackageOnlyAdapter {
    pub fn new(config: PackageOnlyAdapterConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &PackageOnlyAdapterConfig {
        &self.config
    }
}

impl AdapterDefinition for PackageOnlyAdapter {
    type Target = AdaptedCounter;
    type SchemaSupport = AdapterResourceSchemaSupport;

    fn schema_support(&self) -> AdapterResourceSchemaSupport {
        AdapterResourceSchemaSupport::provisional(AdaptedCounter::resource_id())
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        let key = adapted_counter::realization::raw::contract_key(
            ContractVersion::parse("1.0.0").expect("static realization version"),
        );
        ModuleDeclaration::new(provider_module_id).with_provided_contracts(vec![key.declaration()])
    }
}
