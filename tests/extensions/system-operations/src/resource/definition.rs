use fabric::prelude::{
    IntoResourceName, PrimaryResourceContract, ResourceDefinition, ResourceSelection,
    SystemRequires,
};
use fabric_core::{ContractKey, ContractVersionRequirement, ModuleDeclaration};
use fabric_resource::{ResourceError, ResourceId, ResourceSchemaDescriptor};

use crate::{
    SystemBackedResourceContract, TestOperations, resource::runtime::SystemBackedResourceRuntime,
};

#[derive(Clone)]
pub struct SystemBackedResourceConfig {
    pub multiplier: u64,
}

pub struct SystemBackedResource;

impl SystemBackedResource {
    pub fn select(
        name: impl IntoResourceName,
        config: SystemBackedResourceConfig,
    ) -> Result<ResourceSelection<Self>, ResourceError> {
        <Self as ResourceDefinition>::select(name, config)
    }
}

impl PrimaryResourceContract for SystemBackedResource {
    type Contract = SystemBackedResourceContract;

    fn primary_contract_key() -> ContractKey<Self::Contract> {
        crate::system_backed_resource_contract_key()
    }
}

impl ResourceDefinition for SystemBackedResource {
    type Config = SystemBackedResourceConfig;

    fn resource_id() -> ResourceId {
        system_backed_resource_id()
    }

    fn schema() -> ResourceSchemaDescriptor {
        ResourceSchemaDescriptor::provisional(Self::resource_id())
    }

    fn declaration(selection: &ResourceSelection<Self>) -> ModuleDeclaration {
        ModuleDeclaration::new(selection.module_id().clone())
            .with_provided_contracts(vec![
                crate::system_backed_resource_contract_key().declaration(),
            ])
            .with_required_contracts(vec![
                SystemRequires::<TestOperations>::versioned(
                    ContractVersionRequirement::parse("^1.2").expect("static requirement"),
                )
                .declaration()
                .clone(),
            ])
    }

    fn materialize(
        selection: &ResourceSelection<Self>,
    ) -> Option<Box<dyn fabric_core::ModuleRuntime>> {
        Some(Box::new(SystemBackedResourceRuntime::new(
            selection.module_id().clone(),
            selection.config().clone(),
            SystemRequires::<TestOperations>::versioned(
                ContractVersionRequirement::parse("^1.2").expect("static requirement"),
            ),
        )))
    }
}

pub fn system_backed_resource_id() -> ResourceId {
    ResourceId::new("fabric.test.resource.system-backed").expect("static resource id")
}
