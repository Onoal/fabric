use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use fabric::prelude::*;
use fabric_component::{ComponentError, ComponentRuntimeScope};
use fabric_core::{
    ContractId, ContractKey, Health, ModuleBindings, ModuleContract, ModuleDeclaration,
    ModuleError, ModuleId, ModuleRuntime,
};
use fabric_test_component_greeter::{Greeter, GreeterConfig, GreeterInput, GreeterOutput, greeter};

#[derive(Clone)]
struct Audit;
#[derive(Clone)]
struct AuditService;
impl ComponentAugmentationDefinition<Greeter> for Audit {
    type Config = ();
    type Contract = AuditService;
    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(ContractId::new("fabric.test.component.audit").expect("id"))
    }
}
#[derive(Clone)]
struct AuditSupport(Arc<AtomicUsize>);
struct AuditRuntime {
    id: ModuleId,
}
impl ModuleRuntime for AuditRuntime {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &Audit::contract_key(),
            Arc::new(AuditService),
        )])
    }
    fn bind(&mut self, _: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }
    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn stop(&mut self) {}
    fn health(&self) -> Health {
        Health::Healthy
    }
}
impl ComponentAugmentationSupportDefinition<Greeter, Audit> for AuditSupport {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(&self, _: &(), id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(AuditRuntime { id }))
    }
    fn prepare(&self, _: &(), _: &ComponentRuntimeScope) -> Result<(), ComponentError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone)]
struct FailingAudit;
impl ComponentAugmentationSupportDefinition<Greeter, Audit> for FailingAudit {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(&self, _: &(), id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(AuditRuntime { id }))
    }
    fn prepare(&self, _: &(), _: &ComponentRuntimeScope) -> Result<(), ComponentError> {
        Err(ComponentError::Unavailable)
    }
}

#[test]
fn external_component_semantic_prepares_alongside_base_operations() {
    let prepared = Arc::new(AtomicUsize::new(0));
    let built = Fabric::new("fabric.test.component.augmentation")
        .expect("fabric")
        .component(
            Greeter::define(GreeterConfig {})
                .augment::<Audit>(())
                .expect("attach")
                .using(AuditSupport(Arc::clone(&prepared))),
        )
        .build()
        .expect("build");
    assert_eq!(built.manifest().components()[0].operations().len(), 1);
    assert_eq!(
        built.manifest().component_augmentations()[0].component_id(),
        &Greeter::component_id()
    );
    let mut instance = built
        .materialize_named("component-augmentation")
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("components");
    components.materialize::<Greeter>().expect("participate");
    assert_eq!(prepared.load(Ordering::SeqCst), 1);
    let output: GreeterOutput = futures::executor::block_on(components.invoke_external(
        &greeter::operations::greet(),
        GreeterInput { name: "Ada".into() },
    ))
    .expect("invoke");
    assert_eq!(output.message, "hello, Ada");
}

#[test]
fn failed_augmentation_preparation_never_activates_the_component() {
    let built = Fabric::new("fabric.test.component.augmentation.failure")
        .expect("fabric")
        .component(
            Greeter::define(GreeterConfig {})
                .augment::<Audit>(())
                .expect("attach")
                .using(FailingAudit),
        )
        .build()
        .expect("build");
    let mut instance = built
        .materialize_named("component-augmentation-failure")
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("components");
    assert!(components.materialize::<Greeter>().is_err());
}
