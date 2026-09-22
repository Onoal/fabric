use fabric_core::{
    ContractRequirementDeclaration, Health, HostMaterializationRequirement, InstanceRuntimeContext,
    Module, ModuleBindings, ModuleContract, ModuleDeclaration, ModuleError, ModuleId,
    ModuleRuntime, ProvidedContractDeclaration,
};

use super::AdapterDefinition;

pub struct AdapterProviderModule<A>
where
    A: AdapterDefinition,
{
    provider_module_id: ModuleId,
    adapter: A,
    semantic_requirements: Vec<ContractRequirementDeclaration>,
}

impl<A> Clone for AdapterProviderModule<A>
where
    A: AdapterDefinition,
{
    fn clone(&self) -> Self {
        Self {
            provider_module_id: self.provider_module_id.clone(),
            adapter: self.adapter.clone(),
            semantic_requirements: self.semantic_requirements.clone(),
        }
    }
}

impl<A> AdapterProviderModule<A>
where
    A: AdapterDefinition,
{
    pub(crate) fn new(provider_module_id: ModuleId, adapter: A) -> Self {
        Self {
            provider_module_id,
            adapter,
            semantic_requirements: Vec::new(),
        }
    }

    pub fn provider_module_id(&self) -> &ModuleId {
        &self.provider_module_id
    }

    pub fn adapter(&self) -> &A {
        &self.adapter
    }

    /// Attaches semantic Relation declarations to the selected Adapter-owned
    /// live participant. The declarations preserve graph ordering without
    /// creating a semantic Resource/System runtime or forwarding proxy.
    pub(crate) fn with_semantic_requirements(
        mut self,
        requirements: Vec<ContractRequirementDeclaration>,
    ) -> Self {
        self.semantic_requirements = requirements;
        self
    }
}

impl<A> Module for AdapterProviderModule<A>
where
    A: AdapterDefinition,
{
    fn declaration(&self) -> ModuleDeclaration {
        let declaration = self
            .adapter
            .declaration(self.provider_module_id.clone())
            .with_host_requirement(HostMaterializationRequirement::new(
                self.provider_module_id.clone(),
                self.adapter.host_requirement(),
            ));
        let mut required = declaration.required_contracts().to_vec();
        required.extend(self.semantic_requirements.iter().cloned());
        declaration.with_required_contracts(required)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        self.adapter
            .materialize_provider(self.provider_module_id.clone())
            .map(|inner| {
                Box::new(AdapterRequirementsRuntime {
                    inner,
                    semantic_requirements: self.semantic_requirements.clone(),
                }) as Box<dyn ModuleRuntime>
            })
    }
}

struct AdapterRequirementsRuntime {
    inner: Box<dyn ModuleRuntime>,
    semantic_requirements: Vec<ContractRequirementDeclaration>,
}

impl ModuleRuntime for AdapterRequirementsRuntime {
    fn id(&self) -> &ModuleId {
        self.inner.id()
    }

    fn provided_contract_declarations(&self) -> Vec<ProvidedContractDeclaration> {
        self.inner.provided_contract_declarations()
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        let mut required = self.inner.required_contract_declarations();
        required.extend(self.semantic_requirements.iter().cloned());
        required
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        self.inner.export_contracts()
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.inner.bind(bindings)
    }

    fn bind_instance_context(
        &mut self,
        context: &InstanceRuntimeContext,
    ) -> Result<(), ModuleError> {
        self.inner.bind_instance_context(context)
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.inner.initialize()
    }
    fn start(&mut self) -> Result<(), ModuleError> {
        self.inner.start()
    }
    fn stop(&mut self) -> Result<(), ModuleError> {
        self.inner.stop()
    }
    fn health(&self) -> Health {
        self.inner.health()
    }
}
