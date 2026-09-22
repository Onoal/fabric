//! Typed, derived contract providers.
//!
//! A derived provider exports one typed contract assembled from ordinary typed
//! requirements.  It is deliberately a normal Core runtime: composition
//! still owns ordering, binding, and cleanup, while a stateless factory has
//! no independent live machinery to manage.

use std::sync::Arc;

use crate::{
    ContractKey, ContractRequirementDeclaration, Health, InstanceRuntimeContext, ModuleBindings,
    ModuleContract, ModuleError, ModuleId, ModuleRuntime, ProvidedContractDeclaration,
};

/// Factory for a typed contract derived from normal Core requirements.
///
/// Implementations commonly retain typed dependency handles in the exported
/// service.  Core exports before binding, so that service must tolerate calls
/// only after normal composition binding has completed.
pub trait DerivedContractFactory<T>: Send
where
    T: Send + Sync + 'static,
{
    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration>;

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError>;

    fn build(&self) -> Result<T, ModuleError>;
}

/// A normal Core provider whose exported contract is assembled from typed
/// requirements rather than owned concrete machinery.
///
/// It deliberately has default, no-op lifecycle behavior.  A factory that
/// needs live state belongs in a dedicated live provider instead.
pub struct DerivedContractProvider<T, F>
where
    T: Send + Sync + 'static,
    F: DerivedContractFactory<T>,
{
    module_id: ModuleId,
    key: ContractKey<T>,
    factory: F,
}

impl<T, F> DerivedContractProvider<T, F>
where
    T: Send + Sync + 'static,
    F: DerivedContractFactory<T>,
{
    pub fn new(module_id: ModuleId, key: ContractKey<T>, factory: F) -> Self {
        Self {
            module_id,
            key,
            factory,
        }
    }
}

impl<T, F> ModuleRuntime for DerivedContractProvider<T, F>
where
    T: Send + Sync + 'static,
    F: DerivedContractFactory<T>,
{
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<ProvidedContractDeclaration> {
        vec![self.key.declaration()]
    }

    fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
        self.factory.required_contract_declarations()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &self.key,
            Arc::new(self.factory.build()?),
        )])
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.factory.bind(bindings)
    }

    fn bind_instance_context(
        &mut self,
        _context: &InstanceRuntimeContext,
    ) -> Result<(), ModuleError> {
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{DerivedContractFactory, DerivedContractProvider};
    use crate::{
        BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractId, ContractKey,
        ContractRequirement, ContractRequirementDeclaration, Health, InstanceId, ModuleBindings,
        ModuleContract, ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime, module_factory,
    };

    #[derive(Clone)]
    struct Source {
        value: usize,
    }

    #[derive(Clone)]
    struct Derived {
        source: Arc<Mutex<Option<Arc<Source>>>>,
    }

    impl Derived {
        fn value(&self) -> usize {
            self.source
                .lock()
                .expect("source lock")
                .as_ref()
                .expect("source is bound before use")
                .value
                + 1
        }
    }

    #[derive(Clone)]
    struct SourceProvider {
        module_id: ModuleId,
        key: ContractKey<Source>,
    }

    impl ModuleRuntime for SourceProvider {
        fn id(&self) -> &ModuleId {
            &self.module_id
        }

        fn provided_contract_declarations(&self) -> Vec<crate::ProvidedContractDeclaration> {
            vec![self.key.declaration()]
        }

        fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
            Ok(vec![ModuleContract::new(
                &self.key,
                Arc::new(Source { value: 41 }),
            )])
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
    struct DerivedFactory {
        requirement: ContractRequirement<Source>,
        source: Arc<Mutex<Option<Arc<Source>>>>,
    }

    impl DerivedContractFactory<Derived> for DerivedFactory {
        fn required_contract_declarations(&self) -> Vec<ContractRequirementDeclaration> {
            vec![self.requirement.declaration().clone()]
        }

        fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
            let source = bindings
                .resolve(&self.requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?;
            *self.source.lock().expect("source lock") = Some(source);
            Ok(())
        }

        fn build(&self) -> Result<Derived, ModuleError> {
            Ok(Derived {
                source: Arc::clone(&self.source),
            })
        }
    }

    #[derive(Clone)]
    struct Consumer {
        module_id: ModuleId,
        requirement: ContractRequirement<Derived>,
        observed: Arc<Mutex<Option<usize>>>,
        derived: Option<Arc<Derived>>,
    }

    impl ModuleRuntime for Consumer {
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
            self.derived = Some(
                bindings
                    .resolve(&self.requirement)
                    .map_err(|error| ModuleError::new(error.to_string()))?,
            );
            Ok(())
        }

        fn initialize(&mut self) -> Result<(), ModuleError> {
            Ok(())
        }

        fn start(&mut self) -> Result<(), ModuleError> {
            *self.observed.lock().expect("observed lock") = Some(
                self.derived
                    .as_ref()
                    .expect("derived contract bound")
                    .value(),
            );
            Ok(())
        }

        fn stop(&mut self) -> Result<(), ModuleError> {
            Ok(())
        }

        fn health(&self) -> Health {
            Health::Healthy
        }
    }

    #[test]
    fn derived_provider_binds_a_typed_requirement_and_exports_one_derived_contract() {
        let source_key = ContractKey::provisional(
            ContractId::new("fabric.test.derived.source".to_owned()).expect("source id"),
        );
        let derived_key = ContractKey::provisional(
            ContractId::new("fabric.test.derived.result".to_owned()).expect("derived id"),
        );
        let source_requirement = ContractRequirement::provisional(source_key.id().clone());
        let derived_requirement = ContractRequirement::provisional(derived_key.id().clone());
        let source = Arc::new(Mutex::new(None));
        let observed = Arc::new(Mutex::new(None));

        let derived_declaration = ModuleDeclaration::new(
            ModuleId::new("fabric.test.derived.provider".to_owned()).expect("provider id"),
        )
        .with_provided_contracts(vec![derived_key.declaration()])
        .with_required_contracts(vec![source_requirement.declaration().clone()]);

        let composition = CompositionBuilder::new(
            CompositionId::new("fabric.test.derived".to_owned()).expect("composition id"),
        )
        .register_block(
            BlockBuilder::new(BlockId::new("derived".to_owned()).expect("block id"))
                .register_module(SourceProvider {
                    module_id: ModuleId::new("fabric.test.derived.source".to_owned())
                        .expect("source provider id"),
                    key: source_key,
                })
                .register_module(module_factory(derived_declaration, {
                    let source_requirement = source_requirement.clone();
                    let source = Arc::clone(&source);
                    let derived_key = derived_key.clone();
                    move || {
                        DerivedContractProvider::new(
                            ModuleId::new("fabric.test.derived.provider".to_owned())
                                .expect("provider id"),
                            derived_key.clone(),
                            DerivedFactory {
                                requirement: source_requirement.clone(),
                                source: Arc::clone(&source),
                            },
                        )
                    }
                }))
                .register_module(Consumer {
                    module_id: ModuleId::new("fabric.test.derived.consumer".to_owned())
                        .expect("consumer id"),
                    requirement: derived_requirement,
                    observed: Arc::clone(&observed),
                    derived: None,
                })
                .build(),
        )
        .build()
        .expect("typed provider graph composes");

        let mut instance = composition
            .materialize(InstanceId::new("derived".to_owned()).expect("instance id"))
            .expect("typed provider graph materializes");
        instance.start().expect("typed provider graph starts");
        assert_eq!(*observed.lock().expect("observed lock"), Some(42));
        instance.stop().expect("typed provider graph stops");
    }
}
