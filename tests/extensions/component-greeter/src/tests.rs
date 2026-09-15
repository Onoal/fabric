use std::sync::{Arc, Mutex};

use fabric_component::{
    ComponentId, ComponentMaterializer, ComponentRuntimeDefinition, ComponentRuntimeModule,
    InvocationRail, OperationKey, OperationRail,
};
use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractRequirement, Health,
    Instance, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_sdk::ComponentDefinition;
use futures::executor::block_on;

use crate::{
    EmptyComponent, EmptyComponentConfig, Greeter, GreeterConfig, GreeterInput, GreeterOutput,
    PackageComponent, PackageComponentConfig, greeter,
};

#[derive(Clone)]
struct CapturedRails {
    materializer: Arc<ComponentMaterializer>,
    invocation: Arc<InvocationRail>,
    operations: Arc<OperationRail>,
}

type Capture = Arc<Mutex<Option<CapturedRails>>>;

#[derive(Clone)]
struct CaptureModule {
    module_id: ModuleId,
    materializer_requirement: ContractRequirement<ComponentMaterializer>,
    invocation_requirement: ContractRequirement<InvocationRail>,
    operations_requirement: ContractRequirement<OperationRail>,
    capture: Capture,
}

impl CaptureModule {
    fn new(capture: Capture) -> Self {
        Self {
            module_id: ModuleId::new("fabric.test.component.capture").expect("module id"),
            materializer_requirement: ContractRequirement::provisional(
                fabric_component::component_materializer_contract_id(),
            ),
            invocation_requirement: ContractRequirement::provisional(
                fabric_component::invocation_contract_id(),
            ),
            operations_requirement: ContractRequirement::provisional(
                fabric_component::operation_rail_contract_id(),
            ),
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
            self.materializer_requirement.declaration().clone(),
            self.invocation_requirement.declaration().clone(),
            self.operations_requirement.declaration().clone(),
        ]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        *self.capture.lock().expect("capture lock") = Some(CapturedRails {
            materializer: bindings
                .resolve(&self.materializer_requirement)
                .map_err(module_error)?,
            invocation: bindings
                .resolve(&self.invocation_requirement)
                .map_err(module_error)?,
            operations: bindings
                .resolve(&self.operations_requirement)
                .map_err(module_error)?,
        });
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

fn module_error(error: impl std::fmt::Display) -> ModuleError {
    ModuleError::new(error.to_string())
}

fn runtime_fixture(
    components: impl IntoIterator<
        Item = (
            fabric_component::ComponentDeclaration,
            Option<ComponentRuntimeDefinition>,
        ),
    >,
) -> (Instance, CapturedRails) {
    let capture = Arc::new(Mutex::new(None));
    let (declarations, attachments): (Vec<_>, Vec<_>) = components.into_iter().unzip();
    let attachments = attachments.into_iter().flatten().collect::<Vec<_>>();
    let composition = CompositionBuilder::new(
        CompositionId::new("fabric.test.component.macro").expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("fabric.test.component.block").expect("block id"))
            .register_module(
                ComponentRuntimeModule::with_components(declarations, attachments)
                    .expect("component runtime module"),
            )
            .register_module(CaptureModule::new(Arc::clone(&capture)))
            .build(),
    )
    .build()
    .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("fabric.test.component.instance").expect("instance id"))
        .expect("materialize component fixture");
    instance.start().expect("start component fixture");
    let rails = capture
        .lock()
        .expect("capture lock")
        .take()
        .expect("captured rails");
    (instance, rails)
}

fn materialize_component(rails: &CapturedRails, component_id: &ComponentId) {
    rails
        .materializer
        .materialize(component_id)
        .expect("materialize component");
}

fn invoke<I, O>(rails: &CapturedRails, operation: &OperationKey<I, O>, input: I) -> O
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    let context = rails
        .invocation
        .begin_external()
        .expect("external invocation");
    block_on(
        rails
            .operations
            .invoke_with_context(context, operation, input),
    )
    .expect("invoke")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScriptedInput {
    name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScriptedOutput {
    message: String,
}

fabric_sdk::component! {
    pub ScriptedGreeter {
        id: "fabric.test.scripted.greeter";

        config {
            prefix: String;
            suffix: String;
        }

        operations {
            greet {
                id: "fabric.test.scripted.greeter.greet";
                input: ScriptedInput = "fabric.test.scripted.greeter.input";
                output: ScriptedOutput = "fabric.test.scripted.greeter.output";
                handler |input: ScriptedInput| async move {
                    Ok(ScriptedOutput {
                        message: format!("{} {}{}", config.prefix, input.name, config.suffix),
                    })
                };
            }

            part {
                id: "fabric.test.scripted.greeter.part";
                input: ScriptedInput = "fabric.test.scripted.greeter.part.input";
                output: ScriptedOutput = "fabric.test.scripted.greeter.part.output";
                handler |input: ScriptedInput| async move {
                    Ok(ScriptedOutput {
                        message: format!("{} until later, {}{}", config.prefix, input.name, config.suffix),
                    })
                };
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AlphaInput {
    value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AlphaOutput {
    value: u64,
}

fabric_sdk::component! {
    pub AlphaComponent {
        id: "fabric.test.alpha.component";

        config {}

        operations {
            echo {
                id: "fabric.test.alpha.component.echo";
                input: AlphaInput = "fabric.test.alpha.component.input";
                output: AlphaOutput = "fabric.test.alpha.component.output";
                handler |input: AlphaInput| async move {
                    Ok(AlphaOutput {
                        value: input.value + 1,
                    })
                };
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BetaInput {
    value: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BetaOutput {
    value: u64,
}

fabric_sdk::component! {
    pub BetaComponent {
        id: "fabric.test.beta.component";

        config {}

        operations {
            echo {
                id: "fabric.test.beta.component.echo";
                input: BetaInput = "fabric.test.beta.component.input";
                output: BetaOutput = "fabric.test.beta.component.output";
                handler |input: BetaInput| async move {
                    Ok(BetaOutput {
                        value: input.value + 10,
                    })
                };
            }
        }
    }
}

#[test]
fn greeter_component_uses_declared_component_identity() {
    assert_eq!(greeter::component_id().as_str(), "fabric.test.greeter");
    assert_eq!(
        greeter::operations::greet().id().as_str(),
        "fabric.test.greeter.greet"
    );
    assert_eq!(Greeter::component_id().as_str(), "fabric.test.greeter");
    let definition: ComponentRuntimeDefinition = Greeter::define(GreeterConfig {})
        .into_runtime_definition()
        .expect("greeter runtime attachment");
    assert_eq!(
        definition.component_id().as_str(),
        greeter::component_id().as_str()
    );
}

#[test]
fn greeter_operation_uses_declared_operation_identity_metadata() {
    let operation = greeter::operations::greet();

    assert_eq!(operation.id().as_str(), "fabric.test.greeter.greet");
    assert_eq!(
        greeter::operations::greet_input_type_id().as_str(),
        "fabric.test.greeter.input"
    );
    assert_eq!(
        greeter::operations::greet_output_type_id().as_str(),
        "fabric.test.greeter.output"
    );
}

#[test]
fn macro_generated_greeter_invokes_through_component_runtime() {
    let spec = Greeter::define(GreeterConfig {});
    let (mut instance, rails) = runtime_fixture(vec![(
        spec.declaration().clone(),
        spec.into_runtime_definition(),
    )]);
    materialize_component(&rails, &greeter::component_id());

    let output: GreeterOutput = invoke(
        &rails,
        &greeter::operations::greet(),
        GreeterInput {
            name: "kernel".to_owned(),
        },
    );
    assert_eq!(output.message, "hello, kernel");

    instance.stop();
}

#[test]
fn multi_operation_component_captures_config_and_invokes_distinct_handlers() {
    let spec = ScriptedGreeter::define(ScriptedGreeterConfig {
        prefix: "hello".to_owned(),
        suffix: "!".to_owned(),
    });
    let (mut instance, rails) = runtime_fixture(vec![(
        spec.declaration().clone(),
        spec.into_runtime_definition(),
    )]);
    materialize_component(&rails, &scripted_greeter::component_id());

    let greeting: ScriptedOutput = invoke(
        &rails,
        &scripted_greeter::operations::greet(),
        ScriptedInput {
            name: "delta".to_owned(),
        },
    );
    let parting: ScriptedOutput = invoke(
        &rails,
        &scripted_greeter::operations::part(),
        ScriptedInput {
            name: "delta".to_owned(),
        },
    );

    assert_eq!(greeting.message, "hello delta!");
    assert_eq!(parting.message, "hello until later, delta!");
    assert_ne!(
        scripted_greeter::operations::greet().id(),
        scripted_greeter::operations::part().id()
    );

    instance.stop();
}

#[test]
fn multiple_component_macros_can_coexist_in_one_module() {
    let alpha = AlphaComponent::define(AlphaComponentConfig {});
    let beta = BetaComponent::define(BetaComponentConfig {});
    let (mut instance, rails) = runtime_fixture(vec![
        (alpha.declaration().clone(), alpha.into_runtime_definition()),
        (beta.declaration().clone(), beta.into_runtime_definition()),
    ]);
    materialize_component(&rails, &alpha_component::component_id());
    materialize_component(&rails, &beta_component::component_id());

    let alpha: AlphaOutput = invoke(
        &rails,
        &alpha_component::operations::echo(),
        AlphaInput { value: 4 },
    );
    let beta: BetaOutput = invoke(
        &rails,
        &beta_component::operations::echo(),
        BetaInput { value: 4 },
    );

    assert_eq!(alpha.value, 5);
    assert_eq!(beta.value, 14);
    assert_ne!(
        alpha_component::operations::echo().id().as_str(),
        beta_component::operations::echo().id().as_str()
    );

    instance.stop();
}

#[test]
fn package_only_component_defines_identity_config_and_endpoints_without_runtime() {
    let spec = PackageComponent::define(PackageComponentConfig {
        prefix: "hello".to_owned(),
    });

    assert_eq!(spec.config().prefix, "hello");
    assert_eq!(
        PackageComponent::component_id().as_str(),
        "fabric.test.package-component"
    );
    assert_eq!(
        spec.declaration().component_id().as_str(),
        "fabric.test.package-component"
    );
    let operations = spec.declaration().operations();
    assert_eq!(operations.len(), 1);
    assert_eq!(
        operations[0].id().as_str(),
        "fabric.test.package-component.describe"
    );
    assert_eq!(
        operations[0].input_type().as_str(),
        "fabric.test.package-component.describe.input"
    );
    assert_eq!(
        operations[0].output_type().as_str(),
        "fabric.test.package-component.describe.output"
    );
    assert!(spec.into_runtime_definition().is_none());
}

#[test]
fn empty_component_without_operations_is_valid() {
    let spec = EmptyComponent::define(EmptyComponentConfig);

    assert_eq!(
        spec.declaration().component_id().as_str(),
        "fabric.test.empty-component"
    );
    assert!(spec.declaration().operations().is_empty());
    assert!(spec.into_runtime_definition().is_none());
}

#[test]
fn macro_generated_component_exposes_declaration_and_runtime_attachment() {
    let spec = Greeter::define(GreeterConfig {});

    assert_eq!(
        spec.declaration().component_id().as_str(),
        "fabric.test.greeter"
    );
    let operations = spec.declaration().operations();
    assert_eq!(operations.len(), 1);
    assert_eq!(operations[0].id().as_str(), "fabric.test.greeter.greet");
    assert_eq!(
        operations[0].input_type().as_str(),
        "fabric.test.greeter.input"
    );
    assert_eq!(
        operations[0].output_type().as_str(),
        "fabric.test.greeter.output"
    );
    assert!(
        spec.into_runtime_definition().is_some(),
        "generated handlers must still attach a native runtime"
    );
}

#[test]
fn package_only_component_source_contains_no_runtime_obligation() {
    let source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/package_only.rs"))
            .expect("read package-only component witness");
    let code = source
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("//!") || trimmed.starts_with("///"))
        })
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "impl ComponentDefinition for PackageComponent",
        "impl ComponentDefinition for EmptyComponent",
        "fn component_id()",
        "fn declaration()",
        "ComponentDeclaration::new",
        "OperationDefinition::new",
        "PackageComponentConfig",
    ] {
        assert!(
            code.contains(required),
            "package-only component witness must keep declarative anchor {required}"
        );
    }
    for forbidden in [
        "fn prepare",
        "ComponentRuntimeScope",
        "ComponentRuntimeDefinition",
        "Health",
        "ModuleRuntime",
        "Instance",
        "Registry",
        "Control",
        "Readiness",
        "Surface",
        "Reconstruction",
    ] {
        assert!(
            !code.contains(forbidden),
            "package-only component witness must not regain runtime obligation {forbidden}"
        );
    }
}
