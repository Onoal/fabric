use crate::host_materialization::HostMaterializationRequirement;
use crate::module_runtime::ModuleRuntime;
use crate::{ContractRequirementDeclaration, ModuleId, ProvidedContractDeclaration};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleDeclaration {
    module_id: ModuleId,
    provided_contracts: Vec<ProvidedContractDeclaration>,
    required_contracts: Vec<ContractRequirementDeclaration>,
    optional_contracts: Vec<ContractRequirementDeclaration>,
    host_requirement: Option<HostMaterializationRequirement>,
}

impl ModuleDeclaration {
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            provided_contracts: Vec::new(),
            required_contracts: Vec::new(),
            optional_contracts: Vec::new(),
            host_requirement: None,
        }
    }

    pub fn from_runtime(runtime: &dyn ModuleRuntime) -> Self {
        Self::new(runtime.id().clone())
            .with_provided_contracts(runtime.provided_contract_declarations())
            .with_required_contracts(runtime.required_contract_declarations())
            .with_optional_contracts(runtime.optional_contract_declarations())
    }

    pub fn with_provided_contracts(mut self, contracts: Vec<ProvidedContractDeclaration>) -> Self {
        self.provided_contracts = contracts;
        self
    }

    pub fn with_required_contracts(
        mut self,
        contracts: Vec<ContractRequirementDeclaration>,
    ) -> Self {
        self.required_contracts = contracts;
        self
    }

    pub fn with_optional_contracts(
        mut self,
        contracts: Vec<ContractRequirementDeclaration>,
    ) -> Self {
        self.optional_contracts = contracts;
        self
    }

    pub fn with_host_requirement(mut self, requirement: HostMaterializationRequirement) -> Self {
        self.host_requirement = Some(requirement);
        self
    }

    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn provided_contracts(&self) -> &[ProvidedContractDeclaration] {
        &self.provided_contracts
    }

    pub fn required_contracts(&self) -> &[ContractRequirementDeclaration] {
        &self.required_contracts
    }

    pub fn optional_contracts(&self) -> &[ContractRequirementDeclaration] {
        &self.optional_contracts
    }

    pub fn host_requirement(&self) -> Option<&HostMaterializationRequirement> {
        self.host_requirement.as_ref()
    }
}

pub trait Module: Send + Sync {
    fn declaration(&self) -> ModuleDeclaration;

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        None
    }
}

pub struct ModuleFactory<F> {
    declaration: ModuleDeclaration,
    inner: F,
}

pub fn module_factory<F, M>(declaration: ModuleDeclaration, factory: F) -> ModuleFactory<F>
where
    F: Fn() -> M + Send + Sync + 'static,
    M: ModuleRuntime + 'static,
{
    ModuleFactory {
        declaration,
        inner: factory,
    }
}

impl<M> Module for M
where
    M: ModuleRuntime + Clone + Send + Sync + 'static,
{
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::from_runtime(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(self.clone()))
    }
}

impl<F, M> Module for ModuleFactory<F>
where
    F: Fn() -> M + Send + Sync + 'static,
    M: ModuleRuntime + 'static,
{
    fn declaration(&self) -> ModuleDeclaration {
        self.declaration.clone()
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new((self.inner)()))
    }
}
