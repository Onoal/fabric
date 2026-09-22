//! A package-style System defined entirely through public
//! authoring APIs with no runtime materialization.
//!
//! This module intentionally contains:
//!
//! - a `SystemId`
//! - a `SystemSchemaDescriptor`
//! - a `Config` type
//! - declarative contract metadata (`PrimarySystemContract` + `declaration`)
//!
//! and intentionally omits any runtime implementation.
//!
//! Occurrence identity derives only from the `SystemId`, so two selections
//! with different configs share one structural `ModuleId`. A `src/tests.rs`
//! source guard pins this negative space so the package cannot silently
//! regain a runtime obligation.

use fabric::authoring::{PrimarySystemContract, SystemDefinition, SystemSelection};
use fabric_core::{ContractId, ContractKey, ModuleDeclaration};
use fabric_system::{SystemId, SystemSchemaDescriptor};

#[derive(Clone)]
pub struct PackageOnlySystemConfig {
    pub endpoint: String,
}

#[derive(Clone)]
pub struct PackageOnlyCapability;

pub struct PackageOnlySystem;

impl PackageOnlySystem {
    pub fn select(
        config: PackageOnlySystemConfig,
    ) -> Result<SystemSelection<Self>, fabric_system::SystemCompatibilityError> {
        <Self as SystemDefinition>::select(config)
    }
}

impl PrimarySystemContract for PackageOnlySystem {
    type Contract = PackageOnlyCapability;

    fn primary_contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.package-only-system.capability")
                .expect("static capability contract id"),
        )
    }
}

impl SystemDefinition for PackageOnlySystem {
    type Config = PackageOnlySystemConfig;

    fn system_id() -> SystemId {
        SystemId::new("fabric.test.package-only-system").expect("static system id")
    }

    fn schema() -> SystemSchemaDescriptor {
        SystemSchemaDescriptor::provisional(Self::system_id())
    }

    fn declaration(selection: &SystemSelection<Self>) -> ModuleDeclaration {
        ModuleDeclaration::new(selection.module_id().clone())
            .with_provided_contracts(vec![Self::primary_contract_key().declaration()])
    }
}
