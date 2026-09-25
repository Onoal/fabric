use fabric::prelude::*;
use fabric_test_component_greeter::{Greeter, GreeterConfig, GreeterInput, GreeterInstanceApi};

#[test]
fn normal_prelude_supports_the_complete_high_level_component_flow() {
    fn empty_contribution() -> impl IntoFabricContribution {
        FabricContribution::new()
    }

    let composition: Composition = Fabric::new("fabric.test.normal-prelude")
        .expect("fabric")
        .with(empty_contribution())
        .component(Greeter::define(GreeterConfig {}))
        .build()
        .expect("build");
    let _: ComponentDesiredState = composition
        .components()
        .next()
        .expect("component inspection")
        .initial_participation();
    let manifest = composition.manifest();
    assert_eq!(manifest.components().len(), 1);
    assert!(manifest.component_resource_bindings().is_empty());
    assert!(manifest.component_system_bindings().is_empty());
    let mut instance = composition
        .materialize("fabric.test.normal-prelude.instance")
        .expect("materialize");
    let _: InstanceObservation = instance.observe();
    instance.start().expect("start");
    let greeter_component: BoundComponent<'_, Greeter> =
        instance.component::<Greeter>().expect("bound component");
    greeter_component.reconcile().expect("component reconcile");
    let output = futures::executor::block_on(greeter_component.greet(GreeterInput {
        name: "Ada".to_owned(),
    }))
    .expect("typed invocation");
    assert_eq!(output.message, "hello, Ada");
    let components = instance.components().expect("component host");
    components
        .dematerialize::<Greeter>()
        .expect("component dematerialization");
    instance.stop().expect("stop instance");
}

#[test]
fn raw_capabilities_remain_available_through_named_sdk_modules() {
    use fabric::component::invocation::{InvocationRail, OperationRail};
    use fabric::core::{Module, ModuleBindings, ModuleRuntime, ResolvedProviderBinding};

    let _: Option<InvocationRail> = None;
    let _: Option<OperationRail> = None;
    let _: Option<Box<dyn Module>> = None;
    let _: Option<Box<dyn ModuleRuntime>> = None;
    let _: Option<ModuleBindings> = None;
    let _: Option<ResolvedProviderBinding<'_>> = None;
}

#[test]
fn advanced_component_authoring_remains_deliberately_namespaced() {
    use fabric::authoring::component::{
        ComponentDefinition, ComponentRealizationContract, ComponentSpec,
    };
    use fabric::component::advanced::{
        ComponentParticipationPreparation, ComponentParticipationRealization,
        ComponentParticipationScope,
    };

    let _: Option<ComponentSpec<Greeter>> = None;
    let _: Option<ComponentRealizationContract<Greeter>> = None;
    let _: Option<ComponentParticipationRealization> = None;
    let _: Option<ComponentParticipationPreparation> = None;
    let _: Option<&ComponentParticipationScope> = None;
    let _ = <Greeter as ComponentDefinition>::component_id();
}

#[test]
fn component_operator_capabilities_remain_deliberately_namespaced() {
    use fabric::component::operator::{
        ComponentControlRail, ComponentHost, ComponentMaterializer, ComponentReadinessRail,
        ComponentReconstructionRail, ComponentRegistry,
    };

    let _: Option<ComponentHost> = None;
    let _: Option<ComponentRegistry> = None;
    let _: Option<ComponentMaterializer> = None;
    let _: Option<ComponentControlRail> = None;
    let _: Option<ComponentReadinessRail> = None;
    let _: Option<ComponentReconstructionRail> = None;
}
