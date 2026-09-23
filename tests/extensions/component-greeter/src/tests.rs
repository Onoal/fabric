use std::sync::{Arc, Mutex};

use fabric::authoring::component::ComponentDefinition;
use fabric_component::{
    ComponentHostModule, ComponentId, ComponentMaterializer, ComponentParticipationRealization,
    InvocationRail, OperationKey, OperationRail,
};
use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionId, ContractRequirement, Health,
    Instance, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId, ModuleRuntime,
};
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

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

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
            Option<ComponentParticipationRealization>,
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
                ComponentHostModule::with_components(declarations, attachments)
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

fabric::component! {
    pub ScriptedGreeter {
        id: "fabric.test.scripted.greeter";

        config {
            prefix: String;
            suffix: String;
        }

        api {
            fn greet(&self, input: ScriptedInput) -> ScriptedOutput;
            fn part(&self, input: ScriptedInput) -> ScriptedOutput;
        }
        runtime {
            fn greet(&self, input: ScriptedInput) -> ScriptedOutput {
                ScriptedOutput {
                    message: format!("{} {}{}", self.config().prefix, input.name, self.config().suffix),
                }
            }
            fn part(&self, input: ScriptedInput) -> ScriptedOutput {
                ScriptedOutput {
                    message: format!("{} until later, {}{}", self.config().prefix, input.name, self.config().suffix),
                }
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

fabric::component! {
    pub AlphaComponent {
        id: "fabric.test.alpha.component";

        api { fn echo(&self, input: AlphaInput) -> AlphaOutput; }
        runtime {
            fn echo(&self, input: AlphaInput) -> AlphaOutput {
                AlphaOutput { value: input.value + 1 }
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

fabric::component! {
    pub BetaComponent {
        id: "fabric.test.beta.component";

        api { fn echo(&self, input: BetaInput) -> BetaOutput; }
        runtime {
            fn echo(&self, input: BetaInput) -> BetaOutput {
                BetaOutput { value: input.value + 10 }
            }
        }
    }
}

#[test]
fn greeter_component_uses_declared_component_identity() {
    assert_eq!(greeter::component_id().as_str(), "fabric.test.greeter");
    assert_eq!(
        greeter::api::greet().id().as_str(),
        "fabric.test.greeter.api.greet"
    );
    assert_eq!(Greeter::component_id().as_str(), "fabric.test.greeter");
    let definition: ComponentParticipationRealization = Greeter::define(GreeterConfig {})
        .into_self_realization()
        .expect("greeter runtime attachment");
    assert_eq!(
        definition.component_id().as_str(),
        greeter::component_id().as_str()
    );
}

#[test]
fn greeter_api_lowers_to_declared_invocation_identity_metadata() {
    let operation = greeter::api::greet();

    assert_eq!(operation.id().as_str(), "fabric.test.greeter.api.greet");
    assert_eq!(
        greeter::api::greet_input_type_id().as_str(),
        "fabric.test.greeter.api.greet.input"
    );
    assert_eq!(
        greeter::api::greet_output_type_id().as_str(),
        "fabric.test.greeter.api.greet.output"
    );
}

#[test]
fn macro_generated_greeter_invokes_through_component_runtime() {
    let spec = Greeter::define(GreeterConfig {});
    let (mut instance, rails) = runtime_fixture(vec![(
        spec.declaration().clone(),
        spec.into_self_realization(),
    )]);
    materialize_component(&rails, &greeter::component_id());

    let output: GreeterOutput = invoke(
        &rails,
        &greeter::api::greet(),
        GreeterInput {
            name: "kernel".to_owned(),
        },
    );
    assert_eq!(output.message, "hello, kernel");

    instance.stop().expect("stop instance");
}

#[test]
fn multi_api_component_captures_config_and_invokes_distinct_runtime_methods() {
    let spec = ScriptedGreeter::define(ScriptedGreeterConfig {
        prefix: "hello".to_owned(),
        suffix: "!".to_owned(),
    });
    let (mut instance, rails) = runtime_fixture(vec![(
        spec.declaration().clone(),
        spec.into_self_realization(),
    )]);
    materialize_component(&rails, &scripted_greeter::component_id());

    let greeting: ScriptedOutput = invoke(
        &rails,
        &scripted_greeter::api::greet(),
        ScriptedInput {
            name: "delta".to_owned(),
        },
    );
    let parting: ScriptedOutput = invoke(
        &rails,
        &scripted_greeter::api::part(),
        ScriptedInput {
            name: "delta".to_owned(),
        },
    );

    assert_eq!(greeting.message, "hello delta!");
    assert_eq!(parting.message, "hello until later, delta!");
    assert_ne!(
        scripted_greeter::api::greet().id(),
        scripted_greeter::api::part().id()
    );

    instance.stop().expect("stop instance");
}

#[test]
fn multiple_component_macros_can_coexist_in_one_module() {
    let alpha = AlphaComponent::define();
    let beta = BetaComponent::define();
    let (mut instance, rails) = runtime_fixture(vec![
        (alpha.declaration().clone(), alpha.into_self_realization()),
        (beta.declaration().clone(), beta.into_self_realization()),
    ]);
    materialize_component(&rails, &alpha_component::component_id());
    materialize_component(&rails, &beta_component::component_id());

    let alpha: AlphaOutput = invoke(
        &rails,
        &alpha_component::api::echo(),
        AlphaInput { value: 4 },
    );
    let beta: BetaOutput = invoke(&rails, &beta_component::api::echo(), BetaInput { value: 4 });

    assert_eq!(alpha.value, 5);
    assert_eq!(beta.value, 14);
    assert_ne!(
        alpha_component::api::echo().id().as_str(),
        beta_component::api::echo().id().as_str()
    );

    instance.stop().expect("stop instance");
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
    assert!(spec.into_self_realization().is_none());
}

#[test]
fn empty_component_without_api_is_valid() {
    let spec = EmptyComponent::define(EmptyComponentConfig);

    assert_eq!(
        spec.declaration().component_id().as_str(),
        "fabric.test.empty-component"
    );
    assert!(spec.declaration().operations().is_empty());
    assert!(spec.into_self_realization().is_none());
}

#[test]
fn macro_generated_component_exposes_declaration_and_self_realization() {
    let spec = Greeter::define(GreeterConfig {});

    assert_eq!(
        spec.declaration().component_id().as_str(),
        "fabric.test.greeter"
    );
    let operations = spec.declaration().operations();
    assert_eq!(operations.len(), 1);
    assert_eq!(operations[0].id().as_str(), "fabric.test.greeter.api.greet");
    assert_eq!(
        operations[0].input_type().as_str(),
        "fabric.test.greeter.api.greet.input"
    );
    assert_eq!(
        operations[0].output_type().as_str(),
        "fabric.test.greeter.api.greet.output"
    );
    assert!(
        spec.into_self_realization().is_some(),
        "canonical runtime methods must attach a native realization"
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
        "ComponentParticipationScope",
        "ComponentParticipationRealization",
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
