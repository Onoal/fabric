//! The native host catalog is declaration-driven while
//! runtime attachments stay optional.

use std::sync::{Arc, Mutex};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractRequirement, Health,
    Instance, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime,
};

use crate::{
    Component, ComponentDeclaration, ComponentError, ComponentId, ComponentMaterializer,
    ComponentParticipationId, ComponentRegistry, ComponentRuntime, ComponentRuntimeDefinition,
    ComponentRuntimeModule, ComponentRuntimeScope, OperationId, OperationKey, OperationTypeId,
};

const ECHO_A_ID: &str = "fabric.test.catalog.echo-a";
const ECHO_B_ID: &str = "fabric.test.catalog.echo-b";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Echo(String);

#[derive(Clone)]
struct Rails {
    runtime: Arc<ComponentRuntime>,
    materializer: Arc<ComponentMaterializer>,
    registry: Arc<ComponentRegistry>,
}

fn echo_key(operation_id: &str) -> OperationKey<Echo, Echo> {
    OperationKey::new(
        OperationId::new(operation_id).expect("operation id"),
        OperationTypeId::new(format!("{operation_id}.input")).expect("input type id"),
        OperationTypeId::new(format!("{operation_id}.output")).expect("output type id"),
    )
}

fn echo_declaration(component_id: &str, operation_id: &str) -> ComponentDeclaration {
    ComponentDeclaration::new(
        ComponentId::new(component_id).expect("component id"),
        vec![echo_key(operation_id).definition().clone()],
    )
}

fn empty_declaration(component_id: &str) -> ComponentDeclaration {
    ComponentDeclaration::new(
        ComponentId::new(component_id).expect("component id"),
        Vec::new(),
    )
}

fn attachment(
    component_id: &str,
    prepare: impl Fn(&ComponentRuntimeScope) -> Result<Health, ComponentError> + Send + Sync + 'static,
) -> ComponentRuntimeDefinition {
    ComponentRuntimeDefinition::new(
        ComponentId::new(component_id).expect("component id"),
        prepare,
    )
}

fn echo_attachment(component_id: &str, operation_id: &str) -> ComponentRuntimeDefinition {
    let operation_id = operation_id.to_owned();
    attachment(component_id, move |scope| {
        scope.operation(
            echo_key(&operation_id),
            |input: Echo| async move { Ok(input) },
        )?;
        Ok(Health::Healthy)
    })
}

#[derive(Clone)]
struct CaptureModule {
    module_id: ModuleId,
    capture: Arc<Mutex<Option<Rails>>>,
}

impl CaptureModule {
    fn new(capture: Arc<Mutex<Option<Rails>>>) -> Self {
        Self {
            module_id: ModuleId::new("catalog.capture").expect("module id"),
            capture,
        }
    }
}

impl ModuleRuntime for CaptureModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![
            ContractRequirement::<ComponentRuntime>::provisional(
                crate::component_runtime_contract_id(),
            )
            .declaration()
            .clone(),
            ContractRequirement::<ComponentMaterializer>::provisional(
                crate::component_materializer_contract_id(),
            )
            .declaration()
            .clone(),
            ContractRequirement::<ComponentRegistry>::provisional(
                crate::component_registry_contract_id(),
            )
            .declaration()
            .clone(),
        ]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let materializer_requirement = ContractRequirement::<ComponentMaterializer>::provisional(
            crate::component_materializer_contract_id(),
        );
        let runtime_requirement = ContractRequirement::<ComponentRuntime>::provisional(
            crate::component_runtime_contract_id(),
        );
        let registry_requirement = ContractRequirement::<ComponentRegistry>::provisional(
            crate::component_registry_contract_id(),
        );
        *self.capture.lock().expect("capture lock") = Some(Rails {
            runtime: bindings
                .resolve(&runtime_requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?,
            materializer: bindings
                .resolve(&materializer_requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?,
            registry: bindings
                .resolve(&registry_requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?,
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

fn fixture(
    declarations: Vec<ComponentDeclaration>,
    attachments: Vec<ComponentRuntimeDefinition>,
) -> (Instance, Rails) {
    let capture = Arc::new(Mutex::new(None));
    let composition =
        CompositionBuilder::new(CompositionId::new("catalog.fixture").expect("composition id"))
            .register_block(
                BlockBuilder::new(BlockId::new("catalog.block").expect("block id"))
                    .register_module(
                        ComponentRuntimeModule::with_components(declarations, attachments)
                            .expect("component host"),
                    )
                    .register_module(CaptureModule::new(Arc::clone(&capture)))
                    .build(),
            )
            .build()
            .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("catalog.fixture").expect("instance id"))
        .expect("materialize instance");
    instance.start().expect("start instance");
    let rails = capture
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured rails");
    (instance, rails)
}

fn component_id(value: &str) -> ComponentId {
    ComponentId::new(value).expect("component id")
}

#[test]
fn undeclared_operation_registration_fails_and_rolls_back() {
    let captured: Arc<Mutex<Option<ComponentError>>> = Arc::new(Mutex::new(None));
    let captured_for_prepare = Arc::clone(&captured);
    let forged = OperationKey::<Echo, Echo>::new(
        OperationId::new("catalog.forged.echo").expect("operation id"),
        OperationTypeId::new("catalog.forged.echo.input").expect("input type id"),
        OperationTypeId::new("catalog.forged.echo.output").expect("output type id"),
    );
    let capture = Arc::new(Mutex::new(None));
    let composition =
        CompositionBuilder::new(CompositionId::new("catalog.forged").expect("composition id"))
            .register_block(
                BlockBuilder::new(BlockId::new("catalog.forged.block").expect("block id"))
                    .register_module(
                        ComponentRuntimeModule::with_components(
                            vec![empty_declaration("catalog.forged")],
                            vec![attachment("catalog.forged", move |scope| {
                                let error = scope
                                    .operation(
                                        forged.clone(),
                                        |input: Echo| async move { Ok(input) },
                                    )
                                    .expect_err("undeclared endpoint must be rejected");
                                *captured_for_prepare.lock().expect("captured lock") = Some(error);
                                Err(captured_for_prepare
                                    .lock()
                                    .expect("captured lock")
                                    .clone()
                                    .expect("captured error"))
                            })],
                        )
                        .expect("component host"),
                    )
                    .register_module(CaptureModule::new(Arc::clone(&capture)))
                    .build(),
            )
            .build()
            .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("catalog.forged").expect("instance id"))
        .expect("materialize instance");
    instance.start().expect("start instance");

    let rails = capture
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured rails");
    assert_eq!(
        rails
            .materializer
            .materialize(&component_id("catalog.forged"))
            .expect_err("undeclared handler must fail materialization"),
        ComponentError::ComponentRuntimeMaterializationFailed {
            component_id: component_id("catalog.forged"),
            phase: crate::ComponentRuntimeFailurePhase::Prepare,
        }
    );
    assert_eq!(
        captured.lock().expect("captured lock").clone(),
        Some(ComponentError::UndeclaredComponentOperation {
            component_id: component_id("catalog.forged"),
            operation_id: OperationId::new("catalog.forged.echo").expect("operation id"),
        })
    );
    assert_eq!(
        rails.registry.component(&component_id("catalog.forged")),
        Err(ComponentError::UnknownComponent(component_id(
            "catalog.forged"
        ))),
        "failed preparation must roll back without stale authority"
    );
    instance.stop().expect("stop instance");
}

#[test]
fn declared_operation_type_mismatch_fails_registration() {
    let captured: Arc<Mutex<Option<ComponentError>>> = Arc::new(Mutex::new(None));
    let captured_for_prepare = Arc::clone(&captured);
    let forged = OperationKey::<Echo, Echo>::new(
        OperationId::new(ECHO_A_ID).expect("operation id"),
        OperationTypeId::new("catalog.forged.input").expect("input type id"),
        OperationTypeId::new("catalog.forged.output").expect("output type id"),
    );
    let capture = Arc::new(Mutex::new(None));
    let composition =
        CompositionBuilder::new(CompositionId::new("catalog.mismatch").expect("composition id"))
            .register_block(
                BlockBuilder::new(BlockId::new("catalog.mismatch.block").expect("block id"))
                    .register_module(
                        ComponentRuntimeModule::with_components(
                            vec![echo_declaration("catalog.mismatched", ECHO_A_ID)],
                            vec![attachment("catalog.mismatched", move |scope| {
                                let error = scope
                                    .operation(
                                        forged.clone(),
                                        |input: Echo| async move { Ok(input) },
                                    )
                                    .expect_err("mismatched types must be rejected");
                                *captured_for_prepare.lock().expect("captured lock") =
                                    Some(error.clone());
                                Err(error)
                            })],
                        )
                        .expect("component host"),
                    )
                    .register_module(CaptureModule::new(Arc::clone(&capture)))
                    .build(),
            )
            .build()
            .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("catalog.mismatch").expect("instance id"))
        .expect("materialize instance");
    instance.start().expect("start instance");

    let rails = capture
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured rails");
    assert_eq!(
        rails
            .materializer
            .materialize(&component_id("catalog.mismatched"))
            .expect_err("mismatched handler must fail materialization"),
        ComponentError::ComponentRuntimeMaterializationFailed {
            component_id: component_id("catalog.mismatched"),
            phase: crate::ComponentRuntimeFailurePhase::Prepare,
        }
    );
    assert_eq!(
        captured.lock().expect("captured lock").clone(),
        Some(ComponentError::OperationDefinitionTypeMismatch {
            operation_id: OperationId::new(ECHO_A_ID).expect("operation id"),
            expected_input_type: OperationTypeId::new(format!("{ECHO_A_ID}.input"))
                .expect("input type id"),
            actual_input_type: OperationTypeId::new("catalog.forged.input").expect("input type id"),
            expected_output_type: OperationTypeId::new(format!("{ECHO_A_ID}.output"))
                .expect("output type id"),
            actual_output_type: OperationTypeId::new("catalog.forged.output")
                .expect("output type id"),
        })
    );
    instance.stop().expect("stop instance");
}

#[test]
fn mixed_catalog_knows_declarations_and_realizes_only_attached() {
    let (_instance, rails) = fixture(
        vec![
            echo_declaration("catalog.a", ECHO_A_ID),
            echo_declaration("catalog.b", ECHO_B_ID),
        ],
        vec![echo_attachment("catalog.a", ECHO_A_ID)],
    );

    let mut known = rails.materializer.known_component_ids();
    known.sort();
    assert_eq!(
        known,
        vec![component_id("catalog.a"), component_id("catalog.b")]
    );

    let status = rails
        .materializer
        .materialize(&component_id("catalog.a"))
        .expect("attached component materializes");
    assert!(status.is_active());

    assert_eq!(
        rails
            .materializer
            .materialize(&component_id("catalog.b"))
            .expect_err("declaration-only component has no runtime attachment"),
        ComponentError::MissingComponentRuntimeAttachment(component_id("catalog.b"))
    );
}

#[test]
fn unknown_component_is_distinct_from_missing_attachment() {
    let (_instance, rails) = fixture(vec![echo_declaration("catalog.a", ECHO_A_ID)], Vec::new());

    assert_eq!(
        rails
            .materializer
            .materialize(&component_id("catalog.ghost"))
            .expect_err("undeclared component is unknown"),
        ComponentError::UnknownComponent(component_id("catalog.ghost"))
    );
    assert_eq!(
        rails
            .materializer
            .dematerialize(&component_id("catalog.ghost"))
            .expect_err("undeclared component cannot dematerialize"),
        ComponentError::UnknownComponent(component_id("catalog.ghost"))
    );
    assert_eq!(
        rails
            .materializer
            .dematerialize(&component_id("catalog.a"))
            .expect_err("known component without participation is not materialized"),
        ComponentError::ComponentRuntimeNotMaterialized(component_id("catalog.a"))
    );
}

#[test]
fn registry_rejects_undeclared_component_without_minting_participation() {
    let (_instance, rails) = fixture(vec![empty_declaration("catalog.declared")], Vec::new());
    let unknown = Component::bind(component_id("catalog.unknown"), rails.runtime.as_ref());

    assert_eq!(
        rails
            .registry
            .register(unknown, Health::Healthy)
            .expect_err("undeclared component must not participate"),
        ComponentError::UnknownComponent(component_id("catalog.unknown"))
    );
    assert!(rails.registry.components().is_empty());

    let declared = Component::bind(component_id("catalog.declared"), rails.runtime.as_ref());
    let status = rails
        .registry
        .register(declared, Health::Healthy)
        .expect("declared component participates");
    assert_eq!(
        status.participation().participation_id(),
        ComponentParticipationId::new(1),
        "failed registration must not consume participation authority"
    );
}

#[test]
fn declaration_only_component_can_participate_externally() {
    let (_instance, rails) = fixture(vec![empty_declaration("catalog.external")], Vec::new());
    let external = Component::bind(component_id("catalog.external"), rails.runtime.as_ref());
    let participation = rails
        .registry
        .register(external, Health::Healthy)
        .expect("declaration-only component can register")
        .participation()
        .clone();

    let status = rails
        .registry
        .activate(&participation)
        .expect("declaration-only component can activate");
    assert!(status.is_active());
    assert_eq!(
        rails
            .registry
            .component(&component_id("catalog.external"))
            .expect("external participation is retained"),
        status
    );
    assert_eq!(
        rails.materializer.known_component_ids(),
        vec![component_id("catalog.external")]
    );
    assert_eq!(
        rails
            .materializer
            .materialize(&component_id("catalog.external"))
            .expect_err("external participation does not imply native attachment"),
        ComponentError::MissingComponentRuntimeAttachment(component_id("catalog.external"))
    );
}

#[test]
fn fresh_instances_preserve_declarations_without_sharing_live_state() {
    let captures: Arc<Mutex<Vec<Rails>>> = Arc::new(Mutex::new(Vec::new()));
    let composition =
        CompositionBuilder::new(CompositionId::new("catalog.fresh").expect("composition id"))
            .register_block(
                BlockBuilder::new(BlockId::new("catalog.fresh.block").expect("block id"))
                    .register_module(
                        ComponentRuntimeModule::with_components(
                            vec![echo_declaration("catalog.a", ECHO_A_ID)],
                            vec![echo_attachment("catalog.a", ECHO_A_ID)],
                        )
                        .expect("component host"),
                    )
                    .register_module(PushingCaptureModule::new(Arc::clone(&captures)))
                    .build(),
            )
            .build()
            .expect("composition");

    let mut first = composition
        .materialize(InstanceId::new("catalog.first").expect("instance id"))
        .expect("first instance");
    first.start().expect("start first");
    let mut second = composition
        .materialize(InstanceId::new("catalog.second").expect("instance id"))
        .expect("second instance");
    second.start().expect("start second");

    let rails = captures.lock().expect("captures lock").clone();
    assert_eq!(rails.len(), 2);
    for rail in &rails {
        let mut known = rail.materializer.known_component_ids();
        known.sort();
        assert_eq!(known, vec![component_id("catalog.a")]);
        rail.materializer
            .materialize(&component_id("catalog.a"))
            .expect("each instance realizes independently");
    }
    // Live participation does not leak: the second Instance never saw the
    // first Instance's participation, and registries stay per-Instance.
    assert_eq!(
        rails[1]
            .registry
            .component(&component_id("catalog.a"))
            .expect("second participation")
            .component()
            .component_id(),
        &component_id("catalog.a")
    );

    first.stop().expect("stop runtime");
    second.stop().expect("stop runtime");
    assert_eq!(
        rails[0].registry.component(&component_id("catalog.a")),
        Err(ComponentError::UnknownComponent(component_id("catalog.a"))),
        "stopped participation must not leak back into the catalog"
    );
}

#[derive(Clone)]
struct PushingCaptureModule {
    module_id: ModuleId,
    captures: Arc<Mutex<Vec<Rails>>>,
}

impl PushingCaptureModule {
    fn new(captures: Arc<Mutex<Vec<Rails>>>) -> Self {
        Self {
            module_id: ModuleId::new("catalog.pushing-capture").expect("module id"),
            captures,
        }
    }

    fn rails(&self, bindings: &ModuleBindings) -> Result<Rails, ModuleError> {
        let materializer_requirement = ContractRequirement::<ComponentMaterializer>::provisional(
            crate::component_materializer_contract_id(),
        );
        let runtime_requirement = ContractRequirement::<ComponentRuntime>::provisional(
            crate::component_runtime_contract_id(),
        );
        let registry_requirement = ContractRequirement::<ComponentRegistry>::provisional(
            crate::component_registry_contract_id(),
        );
        Ok(Rails {
            runtime: bindings
                .resolve(&runtime_requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?,
            materializer: bindings
                .resolve(&materializer_requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?,
            registry: bindings
                .resolve(&registry_requirement)
                .map_err(|error| ModuleError::new(error.to_string()))?,
        })
    }
}

impl ModuleRuntime for PushingCaptureModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![
            ContractRequirement::<ComponentRuntime>::provisional(
                crate::component_runtime_contract_id(),
            )
            .declaration()
            .clone(),
            ContractRequirement::<ComponentMaterializer>::provisional(
                crate::component_materializer_contract_id(),
            )
            .declaration()
            .clone(),
            ContractRequirement::<ComponentRegistry>::provisional(
                crate::component_registry_contract_id(),
            )
            .declaration()
            .clone(),
        ]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        self.captures
            .lock()
            .expect("captures lock")
            .push(self.rails(bindings)?);
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
