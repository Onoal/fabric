use fabric_core::{ContractId, ModuleId, ModuleRuntime};
use fabric_resource::ResourceId;

use crate::ResourceRegistryError;
use crate::model::{
    ResourceConfigurationFacet, ResourceFacetId, ResourceInspectionFacet, ResourceInterface,
    ResourceInterfaceRole, resource_configuration_facet_id, resource_inspection_facet_id,
};

#[derive(Clone)]
pub struct ResourceDescriptor {
    resource_id: ResourceId,
    module_id: Option<ModuleId>,
    core_interfaces: Vec<ResourceInterface>,
    claimed_interfaces: Vec<ResourceInterface>,
    facets: Vec<ResourceFacetId>,
    configuration: Option<ResourceConfigurationFacet>,
    inspection: Option<ResourceInspectionFacet>,
}

impl ResourceDescriptor {
    pub fn new(resource_id: ResourceId) -> Self {
        Self {
            resource_id,
            module_id: None,
            core_interfaces: Vec::new(),
            claimed_interfaces: Vec::new(),
            facets: Vec::new(),
            configuration: None,
            inspection: None,
        }
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn module_id(&self) -> &ModuleId {
        self.module_id
            .as_ref()
            .expect("resource descriptor must be bound to a module before exposing module id")
    }

    pub fn interfaces(&self) -> &[ResourceInterface] {
        &self.core_interfaces
    }

    pub fn facets(&self) -> &[ResourceFacetId] {
        &self.facets
    }

    pub fn configuration(&self) -> Option<&ResourceConfigurationFacet> {
        self.configuration.as_ref()
    }

    pub fn inspection(&self) -> Option<&ResourceInspectionFacet> {
        self.inspection.as_ref()
    }

    pub fn provided_contracts(&self) -> Vec<&ContractId> {
        self.contracts_by_role(ResourceInterfaceRole::Provides)
    }

    pub fn required_contracts(&self) -> Vec<&ContractId> {
        self.contracts_by_role(ResourceInterfaceRole::Requires)
    }

    pub fn optional_contracts(&self) -> Vec<&ContractId> {
        self.contracts_by_role(ResourceInterfaceRole::Optional)
    }

    pub fn supports_facet(&self, facet_id: &ResourceFacetId) -> bool {
        self.facets.contains(facet_id)
    }

    pub fn with_interface(mut self, interface: ResourceInterface) -> Self {
        if !self.claimed_interfaces.contains(&interface) {
            self.claimed_interfaces.push(interface);
            self.claimed_interfaces.sort();
        }
        self
    }

    pub fn provides(self, contract_id: ContractId) -> Self {
        self.with_interface(ResourceInterface::provided(contract_id))
    }

    pub fn requires(self, contract_id: ContractId) -> Self {
        self.with_interface(ResourceInterface::required(contract_id))
    }

    pub fn optionally_requires(self, contract_id: ContractId) -> Self {
        self.with_interface(ResourceInterface::optional(contract_id))
    }

    pub fn with_facet(mut self, facet_id: ResourceFacetId) -> Self {
        if !self.facets.contains(&facet_id) {
            self.facets.push(facet_id);
            self.facets.sort();
        }
        self
    }

    pub fn with_configuration(mut self, configuration: ResourceConfigurationFacet) -> Self {
        self.configuration = Some(configuration);
        if !self.supports_facet(&resource_configuration_facet_id()) {
            self.facets.push(resource_configuration_facet_id());
            self.facets.sort();
        }
        self
    }

    pub fn with_inspection(mut self, inspection: ResourceInspectionFacet) -> Self {
        self.inspection = Some(inspection);
        if !self.supports_facet(&resource_inspection_facet_id()) {
            self.facets.push(resource_inspection_facet_id());
            self.facets.sort();
        }
        self
    }

    fn contracts_by_role(&self, role: ResourceInterfaceRole) -> Vec<&ContractId> {
        self.core_interfaces
            .iter()
            .filter(|interface| interface.role() == role)
            .map(|interface| interface.contract_id())
            .collect()
    }

    pub(crate) fn bind_to_module(
        &self,
        module: &dyn ModuleRuntime,
        excluded_contracts: &[ContractId],
    ) -> Self {
        let core_interfaces = module
            .provided_contract_declarations()
            .into_iter()
            .map(|declaration| declaration.id().clone())
            .filter(|contract_id| !excluded_contracts.contains(contract_id))
            .map(ResourceInterface::provided)
            .chain(
                module
                    .required_contract_declarations()
                    .into_iter()
                    .map(|declaration| declaration.id().clone())
                    .filter(|contract_id| !excluded_contracts.contains(contract_id))
                    .map(ResourceInterface::required),
            )
            .chain(
                module
                    .optional_contract_declarations()
                    .into_iter()
                    .map(|declaration| declaration.id().clone())
                    .filter(|contract_id| !excluded_contracts.contains(contract_id))
                    .map(ResourceInterface::optional),
            )
            .collect();

        Self {
            resource_id: self.resource_id.clone(),
            module_id: Some(module.id().clone()),
            core_interfaces: sort_unique_interfaces(core_interfaces),
            claimed_interfaces: self.claimed_interfaces.clone(),
            facets: self.facets.clone(),
            configuration: self.configuration.clone(),
            inspection: self.inspection.clone(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ResourceRegistryError> {
        if self.module_id.is_none() {
            return Err(ResourceRegistryError::invalid_input(format!(
                "resource `{}` must be bound to an actual fabric module before registration",
                self.resource_id
            )));
        }
        for interface in &self.claimed_interfaces {
            if !self.core_interfaces.contains(interface) {
                return Err(ResourceRegistryError::InterfaceClaimMismatch {
                    resource_id: self.resource_id.clone(),
                    contract_id: interface.contract_id().as_str().to_owned(),
                    role: interface.role().to_string(),
                });
            }
        }
        if self.configuration.is_none() && self.supports_facet(&resource_configuration_facet_id()) {
            return Err(ResourceRegistryError::invalid_input(format!(
                "resource `{}` declares configuration participation without a configuration boundary",
                self.resource_id
            )));
        }
        if self.inspection.is_none() && self.supports_facet(&resource_inspection_facet_id()) {
            return Err(ResourceRegistryError::invalid_input(format!(
                "resource `{}` declares inspection participation without a safe inspection boundary",
                self.resource_id
            )));
        }
        Ok(())
    }
}

fn sort_unique_interfaces(mut interfaces: Vec<ResourceInterface>) -> Vec<ResourceInterface> {
    interfaces.sort();
    interfaces.dedup();
    interfaces
}
