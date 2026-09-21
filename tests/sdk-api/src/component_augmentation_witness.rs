use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use fabric::prelude::*;
use fabric_component::{
    ComponentError, ComponentRuntimeScope, OperationId, OperationKey, OperationTypeId,
};
use fabric_core::{
    ContractId, ContractKey, ContractRequirement, Health, ModuleBindings, ModuleContract,
    ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
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
struct Trace;
#[derive(Clone)]
struct TraceService;
impl ComponentAugmentationDefinition<Greeter> for Trace {
    type Config = ();
    type Contract = TraceService;
    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(ContractId::new("fabric.test.component.trace").expect("id"))
    }
}

struct QuietComponent;

impl ComponentDefinition for QuietComponent {
    type Config = ();

    fn component_id() -> fabric_component::ComponentId {
        fabric_component::ComponentId::new("fabric.test.component.quiet").expect("component id")
    }

    fn declaration() -> fabric_component::ComponentDeclaration {
        fabric_component::ComponentDeclaration::new(Self::component_id(), Vec::new())
    }
}

impl SelfRealizingComponentDefinition for QuietComponent {
    fn self_realization(_: &()) -> fabric_component::ComponentRuntimeDefinition {
        fabric_component::ComponentRuntimeDefinition::new(Self::component_id(), |_| {
            Ok(Health::Healthy)
        })
    }
}

#[derive(Clone)]
struct QuietAudit;
#[derive(Clone)]
struct QuietAuditService;
impl ComponentAugmentationDefinition<QuietComponent> for QuietAudit {
    type Config = ();
    type Contract = QuietAuditService;
    fn contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(
            ContractId::new("fabric.test.component.quiet.audit").expect("contract"),
        )
    }
}

#[derive(Clone)]
struct MissingAudit {
    id: ModuleId,
    requirement: ContractRequirement<AuditService>,
}

#[derive(Clone)]
struct AuditConsumer {
    id: ModuleId,
    requirement: ComponentAugmentationRequirement<Greeter, Audit>,
    bound: Arc<AtomicUsize>,
}

impl AuditConsumer {
    fn new(
        requirement: ComponentAugmentationRequirement<Greeter, Audit>,
        bound: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            id: ModuleId::new("fabric.test.component.audit.consumer").expect("module id"),
            requirement,
            bound,
        }
    }
}

impl ModuleRuntime for AuditConsumer {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.augmentation().declaration().clone()]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        bindings
            .resolve(self.requirement.augmentation())
            .map_err(|error| ModuleError::new(error.to_string()))?;
        self.bound.fetch_add(1, Ordering::SeqCst);
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
struct TraceConsumer {
    id: ModuleId,
    requirement: ComponentAugmentationRequirement<Greeter, Trace>,
    bound: Arc<AtomicUsize>,
}

impl TraceConsumer {
    fn new(
        requirement: ComponentAugmentationRequirement<Greeter, Trace>,
        bound: Arc<AtomicUsize>,
    ) -> Self {
        Self {
            id: ModuleId::new("fabric.test.component.trace.consumer").expect("module id"),
            requirement,
            bound,
        }
    }
}

impl ModuleRuntime for TraceConsumer {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.augmentation().declaration().clone()]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }
    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        bindings
            .resolve(self.requirement.augmentation())
            .map_err(|error| ModuleError::new(error.to_string()))?;
        self.bound.fetch_add(1, Ordering::SeqCst);
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
impl ModuleRuntime for MissingAudit {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.requirement.declaration().clone()]
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
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
    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn health(&self) -> Health {
        Health::Healthy
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
    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn health(&self) -> Health {
        Health::Healthy
    }
}

struct TraceRuntime {
    id: ModuleId,
}

struct QuietAuditRuntime {
    id: ModuleId,
}
impl ModuleRuntime for QuietAuditRuntime {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &QuietAudit::contract_key(),
            Arc::new(QuietAuditService),
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
    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
    fn health(&self) -> Health {
        Health::Healthy
    }
}
impl ModuleRuntime for TraceRuntime {
    fn id(&self) -> &ModuleId {
        &self.id
    }
    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &Trace::contract_key(),
            Arc::new(TraceService),
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
    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
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
struct TraceSupport(Arc<Mutex<Vec<fabric_component::ComponentParticipationId>>>);
impl ComponentAugmentationSupportDefinition<Greeter, Trace> for TraceSupport {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(&self, _: &(), id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(TraceRuntime { id }))
    }
    fn prepare(&self, _: &(), scope: &ComponentRuntimeScope) -> Result<(), ComponentError> {
        self.0
            .lock()
            .expect("trace participations")
            .push(scope.participation().participation_id());
        Ok(())
    }
}

#[derive(Clone)]
struct QuietAuditSupport(Arc<AtomicUsize>);
impl ComponentAugmentationSupportDefinition<QuietComponent, QuietAudit> for QuietAuditSupport {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(&self, _: &(), id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(QuietAuditRuntime { id }))
    }
    fn prepare(&self, _: &(), _: &ComponentRuntimeScope) -> Result<(), ComponentError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone)]
struct AuditingSupport(Arc<Mutex<Vec<fabric_component::ComponentParticipationId>>>);
impl ComponentAugmentationSupportDefinition<Greeter, Audit> for AuditingSupport {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(&self, _: &(), id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(AuditRuntime { id }))
    }
    fn prepare(&self, _: &(), scope: &ComponentRuntimeScope) -> Result<(), ComponentError> {
        self.0
            .lock()
            .expect("audit participations")
            .push(scope.participation().participation_id());
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

#[derive(Clone)]
struct TeardownAuditSupport(Arc<AtomicUsize>);
impl ComponentAugmentationSupportDefinition<Greeter, Audit> for TeardownAuditSupport {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(&self, _: &(), id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(AuditRuntime { id }))
    }
    fn prepare(&self, _: &(), _: &ComponentRuntimeScope) -> Result<(), ComponentError> {
        Ok(())
    }
    fn prepare_with_teardown(
        &self,
        _: &(),
        _: &ComponentRuntimeScope,
    ) -> Result<ComponentAugmentationRuntimePreparation, ComponentError> {
        let teardown_count = Arc::clone(&self.0);
        Ok(ComponentAugmentationRuntimePreparation::with_teardown(
            move || {
                teardown_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
        ))
    }
}

#[derive(Clone)]
struct UndeclaredOperationAudit;
impl ComponentAugmentationSupportDefinition<Greeter, Audit> for UndeclaredOperationAudit {
    fn declaration(&self, id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(id)
    }
    fn materialize(&self, _: &(), id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(AuditRuntime { id }))
    }
    fn prepare(&self, _: &(), scope: &ComponentRuntimeScope) -> Result<(), ComponentError> {
        scope.operation(
            OperationKey::<(), ()>::new(
                OperationId::new("fabric.test.component.audit.undeclared").expect("id"),
                OperationTypeId::new("fabric.test.component.audit.undeclared.input")
                    .expect("input type"),
                OperationTypeId::new("fabric.test.component.audit.undeclared.output")
                    .expect("output type"),
            ),
            |_| async { Ok(()) },
        )
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
fn third_party_component_augmentation_teardown_is_participation_scoped() {
    let teardown_count = Arc::new(AtomicUsize::new(0));
    let built = Fabric::new("fabric.test.component.augmentation.teardown")
        .expect("fabric")
        .component(
            Greeter::define(GreeterConfig {})
                .augment::<Audit>(())
                .expect("attach")
                .using(TeardownAuditSupport(Arc::clone(&teardown_count))),
        )
        .build()
        .expect("build");
    let mut instance = built
        .materialize_named("component-augmentation-teardown")
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("components");
    components.materialize::<Greeter>().expect("participate");
    components
        .dematerialize::<Greeter>()
        .expect("dematerialize");
    assert_eq!(teardown_count.load(Ordering::SeqCst), 1);
    instance.stop().expect("stop");
    assert_eq!(teardown_count.load(Ordering::SeqCst), 1);
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
    assert!(
        futures::executor::block_on(components.invoke_external(
            &greeter::operations::greet(),
            GreeterInput { name: "Ada".into() },
        ))
        .is_err()
    );
}

#[test]
fn augmentation_cannot_register_an_operation_absent_from_base_declaration() {
    let built = Fabric::new("fabric.test.component.augmentation.undeclared-operation")
        .expect("fabric")
        .component(
            Greeter::define(GreeterConfig {})
                .augment::<Audit>(())
                .expect("attach")
                .using(UndeclaredOperationAudit),
        )
        .build()
        .expect("build");
    assert_eq!(built.manifest().components()[0].operations().len(), 1);
    let mut instance = built
        .materialize_named("component-augmentation-undeclared-operation")
        .expect("materialize");
    instance.start().expect("start");
    assert!(
        instance
            .components()
            .expect("components")
            .materialize::<Greeter>()
            .is_err()
    );
}

#[test]
fn bare_attachment_is_manifest_truth_but_does_not_fabricate_x() {
    let attachment = Greeter::define(GreeterConfig {})
        .augment::<Audit>(())
        .expect("attach");
    let error = Fabric::new("fabric.test.component.augmentation.missing")
        .expect("fabric")
        .component(attachment)
        .block("missing-audit", |block| {
            block.module(MissingAudit {
                id: ModuleId::new("fabric.test.component.audit.missing").expect("id"),
                requirement: ContractRequirement::provisional(Audit::contract_key().id().clone()),
            })
        })
        .expect("block")
        .build()
        .expect_err("no X provider");
    assert!(matches!(
        error,
        FabricBuildError::Composition(CompositionError::MissingProvider { .. })
    ));
}

#[test]
fn multiple_semantics_prepare_the_same_component_participation() {
    let audits = Arc::new(Mutex::new(Vec::new()));
    let traces = Arc::new(Mutex::new(Vec::new()));
    let augmentations = Greeter::define(GreeterConfig {})
        .augment::<Audit>(())
        .expect("attach audit")
        .using(AuditingSupport(Arc::clone(&audits)))
        .into_set()
        .augment::<Trace>(())
        .expect("attach trace")
        .using(TraceSupport(Arc::clone(&traces)));
    let built = Fabric::new("fabric.test.component.augmentation.multiple")
        .expect("fabric")
        .component(augmentations)
        .build()
        .expect("build");
    assert_eq!(built.manifest().component_augmentations().len(), 2);
    assert!(
        built
            .manifest()
            .component_augmentations()
            .iter()
            .all(|entry| entry.component_id() == &Greeter::component_id())
    );
    assert_eq!(built.manifest().components()[0].operations().len(), 1);

    let mut instance = built
        .materialize_named("component-augmentation-multiple")
        .expect("materialize");
    instance.start().expect("start");
    instance
        .components()
        .expect("components")
        .materialize::<Greeter>()
        .expect("participate");
    let audit = audits.lock().expect("audits");
    let trace = traces.lock().expect("traces");
    assert_eq!(audit.len(), 1);
    assert_eq!(trace.len(), 1);
    assert_eq!(audit[0], trace[0]);
}

#[test]
fn zero_operation_component_can_receive_external_semantic_support() {
    let prepared = Arc::new(AtomicUsize::new(0));
    let built = Fabric::new("fabric.test.component.augmentation.zero-operation")
        .expect("fabric")
        .component(
            ComponentSpec::<QuietComponent>::self_realizing(())
                .expect("self realization")
                .augment::<QuietAudit>(())
                .expect("attach")
                .using(QuietAuditSupport(Arc::clone(&prepared))),
        )
        .build()
        .expect("build");
    assert!(built.manifest().components()[0].operations().is_empty());
    let mut instance = built
        .materialize_named("component-augmentation-zero-operation")
        .expect("materialize");
    instance.start().expect("start");
    instance
        .components()
        .expect("components")
        .materialize::<QuietComponent>()
        .expect("participate");
    assert_eq!(prepared.load(Ordering::SeqCst), 1);
}

#[test]
fn alternate_supports_preserve_the_same_semantic_contract() {
    let first = Arc::new(AtomicUsize::new(0));
    let second = Arc::new(Mutex::new(Vec::new()));
    let first_built = Fabric::new("fabric.test.component.augmentation.support-one")
        .expect("fabric")
        .component(
            Greeter::define(GreeterConfig {})
                .augment::<Audit>(())
                .expect("attach")
                .using(AuditSupport(Arc::clone(&first))),
        )
        .build()
        .expect("first build");
    let second_built = Fabric::new("fabric.test.component.augmentation.support-two")
        .expect("fabric")
        .component(
            Greeter::define(GreeterConfig {})
                .augment::<Audit>(())
                .expect("attach")
                .using(AuditingSupport(Arc::clone(&second))),
        )
        .build()
        .expect("second build");
    assert_eq!(
        first_built.manifest().component_augmentations()[0].contract_id(),
        second_built.manifest().component_augmentations()[0].contract_id(),
    );

    let mut first_instance = first_built
        .materialize_named("component-augmentation-support-one")
        .expect("first materialize");
    first_instance.start().expect("first start");
    first_instance
        .components()
        .expect("first components")
        .materialize::<Greeter>()
        .expect("first participation");
    let mut second_instance = second_built
        .materialize_named("component-augmentation-support-two")
        .expect("second materialize");
    second_instance.start().expect("second start");
    second_instance
        .components()
        .expect("second components")
        .materialize::<Greeter>()
        .expect("second participation");
    assert_eq!(first.load(Ordering::SeqCst), 1);
    assert_eq!(second.lock().expect("second participations").len(), 1);
}

#[test]
fn multi_augmentation_authoring_retains_typed_requirements_for_each_semantic() {
    let audit_prepared = Arc::new(AtomicUsize::new(0));
    let trace_prepared = Arc::new(Mutex::new(Vec::new()));
    let audit = Greeter::define(GreeterConfig {})
        .augment::<Audit>(())
        .expect("attach audit")
        .using(AuditSupport(Arc::clone(&audit_prepared)));
    let audit_requirement = audit.requirement();
    let trace = audit
        .into_set()
        .augment::<Trace>(())
        .expect("attach trace")
        .using(TraceSupport(Arc::clone(&trace_prepared)));
    let trace_requirement = trace.requirement();
    assert_eq!(
        audit_requirement.target_component_id(),
        &Greeter::component_id()
    );
    assert_eq!(
        trace_requirement.target_component_id(),
        &Greeter::component_id()
    );
    assert_ne!(
        audit_requirement.augmentation().declaration().id(),
        trace_requirement.augmentation().declaration().id(),
    );

    let audit_bound = Arc::new(AtomicUsize::new(0));
    let trace_bound = Arc::new(AtomicUsize::new(0));
    let audit_consumer = AuditConsumer::new(audit_requirement.clone(), Arc::clone(&audit_bound));
    let trace_consumer = TraceConsumer::new(trace_requirement.clone(), Arc::clone(&trace_bound));
    let audit_selection = audit_requirement.provider_selection(audit_consumer.id.clone());
    let trace_selection = trace_requirement.provider_selection(trace_consumer.id.clone());
    let built = Fabric::new("fabric.test.component.augmentation.requirements")
        .expect("fabric")
        .component(trace)
        .block("audit-consumer", |block| block.module(audit_consumer))
        .expect("audit block")
        .block("trace-consumer", |block| block.module(trace_consumer))
        .expect("trace block")
        .select_provider(audit_selection)
        .select_provider(trace_selection)
        .build()
        .expect("build");
    assert_eq!(built.manifest().component_augmentations().len(), 2);
    let mut instance = built
        .materialize_named("component-augmentation-requirements")
        .expect("materialize");
    instance.start().expect("start");
    assert_eq!(audit_bound.load(Ordering::SeqCst), 1);
    assert_eq!(trace_bound.load(Ordering::SeqCst), 1);
}

#[test]
fn bare_attachment_has_no_supported_requirement_while_later_support_remains_usable() {
    let trace_prepared = Arc::new(Mutex::new(Vec::new()));
    let supported_trace = Greeter::define(GreeterConfig {})
        .augment::<Audit>(())
        .expect("attach bare audit")
        .into_set()
        .augment::<Trace>(())
        .expect("attach trace")
        .using(TraceSupport(Arc::clone(&trace_prepared)));
    let trace_requirement = supported_trace.requirement();
    let trace_bound = Arc::new(AtomicUsize::new(0));
    let trace_consumer = TraceConsumer::new(trace_requirement.clone(), Arc::clone(&trace_bound));
    let trace_selection = trace_requirement.provider_selection(trace_consumer.id.clone());
    let built = Fabric::new("fabric.test.component.augmentation.bare-and-supported")
        .expect("fabric")
        .component(supported_trace)
        .block("trace-consumer", |block| block.module(trace_consumer))
        .expect("trace block")
        .select_provider(trace_selection)
        .build()
        .expect("build");
    assert_eq!(built.manifest().component_augmentations().len(), 2);
    assert!(
        built
            .manifest()
            .component_augmentations()
            .iter()
            .any(|entry| entry.contract_id() == Audit::contract_key().id())
    );
    assert!(
        built
            .manifest()
            .component_augmentations()
            .iter()
            .any(|entry| entry.contract_id() == Trace::contract_key().id())
    );
    let mut instance = built
        .materialize_named("component-augmentation-bare-and-supported")
        .expect("materialize");
    instance.start().expect("start");
    assert_eq!(trace_bound.load(Ordering::SeqCst), 1);
}
