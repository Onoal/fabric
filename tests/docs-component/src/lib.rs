#[cfg(test)]
use fabric::authoring::component::ComponentDefinition;
use fabric::component::invocation::InvocationOrigin;
#[allow(unused_imports)]
use fabric::prelude::*;
#[cfg(test)]
use fabric_test_component_greeter::{
    EmptyComponent as DeclarationOnlyComponent,
    EmptyComponentConfig as DeclarationOnlyComponentConfig,
};
use fabric_test_resource_counter::DirectCounter;
#[cfg(test)]
use fabric_test_resource_counter::DirectCounterConfig;
use fabric_test_system_operations::TestOperations;
#[cfg(test)]
use fabric_test_system_operations::TestOperationsConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
struct OpenInput {
    exists: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Document {
    title: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum DocumentError {
    NotFound,
}

fabric::component! {
    pub DocumentComponent {
        id: "example.document-component";
        operations {
            open {
                id: "example.document-component.open";
                input: OpenInput = "example.document-component.open.input";
                output: Result<Document, DocumentError> = "example.document-component.open.outcome";
                context: invocation;
                handler |context, input: OpenInput| async move {
                    assert_eq!(context.root_origin(), &InvocationOrigin::External);
                    if input.exists { Ok(Ok(Document { title: "Fabric".to_owned() })) }
                    else { Ok(Err(DocumentError::NotFound)) }
                };
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DependencyOutput {
    value: u64,
}

fabric::component! {
    pub DependencyComponent {
        id: "example.dependency-component";
        config { multiplier: u64; }
        requires { counter: DirectCounter(provisional); }
        system { operations: TestOperations(version = "^1"); }
        operations {
            observe {
                id: "example.dependency-component.observe";
                input: () = "example.dependency-component.observe.input";
                output: DependencyOutput = "example.dependency-component.observe.output";
                context: invocation;
                handler |context, dependencies, _input: ()| async move {
                    assert_eq!(context.root_origin(), &InvocationOrigin::External);
                    Ok(DependencyOutput {
                        value: dependencies.counter.current_value().value()
                            + dependencies.operations.current_marker().value()
                            + config.multiplier,
                    })
                };
            }
        }
    }
}

fabric::component! {
    pub ZeroOperationComponent {
        id: "example.zero-operation-component";
        operations {}
    }
}

#[test]
fn typed_resource_and_system_dependencies_reach_a_context_aware_handler() {
    let counter = DirectCounter::select("primary", DirectCounterConfig { value: 5 })
        .expect("Resource selection");
    let operations =
        TestOperations::select(TestOperationsConfig::new(7, 1)).expect("System selection");
    let built = Fabric::new("example.component.dependencies")
        .expect("CompositionId")
        .resource(counter.clone())
        .system(operations.clone())
        .component(
            DependencyComponent::define(DependencyComponentConfig { multiplier: 2 })
                .select_resource_provider(&counter)
                .select_system_provider(&operations),
        )
        .build()
        .expect("valid dependencies");
    let mut instance = built
        .materialize_named("example.component.dependencies.local")
        .expect("Instance");
    let instance_id = instance.instance_id().clone();
    let generation = instance.generation();
    instance.start().expect("start");
    let components = instance
        .components()
        .expect("ComponentInstanceBinding host");
    components
        .materialize::<DependencyComponent>()
        .expect("participation");
    let output = futures::executor::block_on(
        components.invoke_external(&dependency_component::operations::observe(), ()),
    )
    .expect("invoke");
    assert_eq!(output, DependencyOutput { value: 14 });
    assert_eq!(instance.instance_id(), &instance_id);
    assert_eq!(instance.generation(), generation);
    instance.stop().expect("stop instance");
}

#[test]
fn declaration_only_component_is_valid_but_has_no_native_attachment() {
    let built = Fabric::new("example.component.declaration-only")
        .expect("CompositionId")
        .component(DeclarationOnlyComponent::define(
            DeclarationOnlyComponentConfig,
        ))
        .build()
        .expect("declaration-only ComponentInstanceBinding is valid");
    assert_eq!(built.manifest().components()[0].operations().len(), 0);
    let mut instance = built
        .materialize_named("example.component.declaration-only.local")
        .expect("Instance");
    instance.start().expect("start");
    let error = instance
        .components()
        .expect("ComponentInstanceBinding host")
        .materialize::<DeclarationOnlyComponent>()
        .expect_err("attachment is absent");
    assert!(matches!(
        error,
        fabric::component::ComponentError::MissingComponentParticipationRealization(_)
    ));
    instance.stop().expect("stop instance");
}

#[test]
fn component_operations_context_domain_output_and_participation_are_distinct() {
    let built = Fabric::new("example.component")
        .expect("CompositionId")
        .component(DocumentComponent::define(DocumentComponentConfig {}))
        .component(ZeroOperationComponent::define(
            ZeroOperationComponentConfig {},
        ))
        .build()
        .expect("declarations are valid");
    assert_eq!(built.manifest().components().len(), 2);
    assert!(built.manifest().components()[1].operations().is_empty());
    let mut instance = built
        .materialize_named("example.component.local")
        .expect("Instance");
    instance.start().expect("start");
    let components = instance
        .components()
        .expect("ComponentInstanceBinding host");
    components
        .materialize::<DocumentComponent>()
        .expect("participation");
    let outcome = futures::executor::block_on(components.invoke_external(
        &document_component::operations::open(),
        OpenInput { exists: false },
    ))
    .expect("outer runtime result");
    assert_eq!(outcome, Err(DocumentError::NotFound));
    components
        .dematerialize::<DocumentComponent>()
        .expect("dematerialize");
    instance.stop().expect("stop instance");
}
