//! A package-style Resource defined entirely through public
//! authoring APIs with no runtime materialization.
//!
//! This module intentionally contains:
//!
//! - a `ResourceId`
//! - a `ResourceSchemaDescriptor`
//! - a `Config` type
//! - declarative contract metadata (`PrimaryResourceContract` + `declaration`)
//!
//! and intentionally omits:
//!
//! - `ModuleRuntime`
//! - `initialize` / `start` / `stop` / `health`
//! - any `Instance` or `Composition` requirement merely to define and select it
//!
//! A `src/tests.rs` source guard pins this negative space so the package
//! cannot silently regain a runtime obligation.

use fabric_core::{ContractKey, ModuleDeclaration};
use fabric_resource::{ResourceError, ResourceId, ResourceSchemaDescriptor};
use fabric_sdk::prelude::{
    IntoResourceName, PrimaryResourceContract, ResourceDefinition, ResourceSelection,
};

#[derive(Clone)]
pub struct PackageOnlyConfig {
    pub endpoint: String,
}

#[derive(Clone)]
pub struct PackageOnlyCapability;

pub struct PackageOnlyResource;

impl PackageOnlyResource {
    pub fn select(
        name: impl IntoResourceName,
        config: PackageOnlyConfig,
    ) -> Result<ResourceSelection<Self>, ResourceError> {
        <Self as ResourceDefinition>::select(name, config)
    }
}

impl PrimaryResourceContract for PackageOnlyResource {
    type Contract = PackageOnlyCapability;

    fn primary_contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            fabric_core::ContractId::new("fabric.test.package-only.capability")
                .expect("static capability contract id"),
        )
    }
}

impl ResourceDefinition for PackageOnlyResource {
    type Config = PackageOnlyConfig;

    fn resource_id() -> ResourceId {
        ResourceId::new("fabric.test.package-only").expect("static resource id")
    }

    fn schema() -> ResourceSchemaDescriptor {
        ResourceSchemaDescriptor::provisional(Self::resource_id())
    }

    fn declaration(selection: &ResourceSelection<Self>) -> ModuleDeclaration {
        ModuleDeclaration::new(selection.module_id().clone())
            .with_provided_contracts(vec![Self::primary_contract_key().declaration()])
    }
}
