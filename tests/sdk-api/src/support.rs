use std::sync::{Arc, Mutex};

use fabric::prelude::*;
use fabric::{core::*, ids::*};

#[derive(Clone)]
pub struct NotesContract {
    port: Arc<dyn NotesPort>,
}

impl NotesContract {
    pub fn new(port: Arc<dyn NotesPort>) -> Self {
        Self { port }
    }

    pub fn read(&self) -> String {
        self.port.read()
    }
}

pub trait NotesPort: Send + Sync {
    fn read(&self) -> String;
}

#[derive(Clone)]
pub struct StaticNotes(pub String);

impl NotesPort for StaticNotes {
    fn read(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedResolution {
    pub provider: ModuleId,
    pub identity: ContractIdentity,
    pub value: String,
}

pub type NotesCapture = Arc<Mutex<Option<CapturedResolution>>>;

pub fn notes_contract_id() -> ContractId {
    contract("fabric.test.sdk.notes").expect("static notes contract id")
}

#[derive(Clone)]
pub struct VersionedNotesProvider {
    module_id: ModuleId,
    exports: Vec<(ContractKey<NotesContract>, String)>,
}

impl VersionedNotesProvider {
    pub fn new(module_id: &str, exports: Vec<(ContractKey<NotesContract>, String)>) -> Self {
        Self {
            module_id: module(module_id).expect("module id"),
            exports,
        }
    }
}

impl ModuleRuntime for VersionedNotesProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<ProvidedContractDeclaration> {
        self.exports
            .iter()
            .map(|(key, _)| key.declaration())
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(self
            .exports
            .iter()
            .map(|(key, value)| {
                ModuleContract::new(
                    key,
                    Arc::new(NotesContract::new(Arc::new(StaticNotes(value.clone())))),
                )
            })
            .collect())
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone)]
pub struct NotesConsumer {
    module_id: ModuleId,
    requirement: ContractRequirement<NotesContract>,
    capture: NotesCapture,
}

impl NotesConsumer {
    pub fn new(
        module_id: &str,
        requirement: ContractRequirement<NotesContract>,
        capture: NotesCapture,
    ) -> Self {
        Self {
            module_id: module(module_id).expect("module id"),
            requirement,
            capture,
        }
    }
}

impl ModuleRuntime for NotesConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let resolved: ResolvedContract<NotesContract> = bindings
            .resolve_with_provider(&self.requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        *self.capture.lock().expect("capture lock") = Some(CapturedResolution {
            provider: resolved.provider().clone(),
            identity: resolved.identity().clone(),
            value: resolved.value().read(),
        });
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedRequirementMismatch {
    pub module_id: ModuleId,
    pub contract_id: ContractId,
    pub declared_compatibility: ContractCompatibilityRequirement,
    pub requested_compatibility: ContractCompatibilityRequirement,
}

pub type RequirementMismatchCapture = Arc<Mutex<Option<CapturedRequirementMismatch>>>;

#[derive(Clone)]
pub struct MismatchedNotesConsumer {
    module_id: ModuleId,
    declared_requirement: ContractRequirement<NotesContract>,
    bind_requirement: ContractRequirement<NotesContract>,
    capture: RequirementMismatchCapture,
}

impl MismatchedNotesConsumer {
    pub fn new(
        module_id: &str,
        declared_requirement: ContractRequirement<NotesContract>,
        bind_requirement: ContractRequirement<NotesContract>,
        capture: RequirementMismatchCapture,
    ) -> Self {
        Self {
            module_id: module(module_id).expect("module id"),
            declared_requirement,
            bind_requirement,
            capture,
        }
    }
}

impl ModuleRuntime for MismatchedNotesConsumer {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        vec![self.declared_requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let error = match bindings.resolve_with_provider(&self.bind_requirement) {
            Ok(_) => panic!("mismatched requirement must not resolve"),
            Err(error) => error,
        };
        let CompositionError::RequirementDeclarationMismatch {
            module_id,
            contract_id,
            declared_compatibility,
            requested_compatibility,
        } = error
        else {
            panic!("expected requirement declaration mismatch")
        };
        *self.capture.lock().expect("capture lock") = Some(CapturedRequirementMismatch {
            module_id,
            contract_id,
            declared_compatibility,
            requested_compatibility,
        });
        Err(ModuleError::new("sdk requirement declaration mismatch"))
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}
