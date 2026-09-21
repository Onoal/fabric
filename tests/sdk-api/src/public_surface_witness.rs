use fabric::prelude::*;
use fabric_test_component_greeter::{Greeter, GreeterConfig, GreeterInput, greeter};

#[test]
fn normal_prelude_supports_the_complete_high_level_component_flow() {
    let built = Fabric::new("fabric.test.normal-prelude")
        .expect("fabric")
        .component(Greeter::define(GreeterConfig {}))
        .build()
        .expect("build");
    let manifest = built.manifest();
    assert_eq!(manifest.components().len(), 1);
    assert!(manifest.component_resource_bindings().is_empty());
    assert!(manifest.component_system_bindings().is_empty());
    let mut instance = built
        .materialize_named("fabric.test.normal-prelude.instance")
        .expect("materialize");
    instance.start().expect("start");
    let components = instance.components().expect("component host");
    components
        .materialize::<Greeter>()
        .expect("component materialization");
    let output = futures::executor::block_on(components.invoke_external(
        &greeter::operations::greet(),
        GreeterInput {
            name: "Ada".to_owned(),
        },
    ))
    .expect("typed invocation");
    assert_eq!(output.message, "hello, Ada");
    components
        .dematerialize::<Greeter>()
        .expect("component dematerialization");
    instance.stop().expect("stop instance");
}

#[test]
fn raw_capabilities_remain_available_through_named_sdk_modules() {
    use fabric::component::{InvocationRail, OperationRail};
    use fabric::core::{Module, ModuleBindings, ModuleRuntime};

    let _: Option<InvocationRail> = None;
    let _: Option<OperationRail> = None;
    let _: Option<Box<dyn Module>> = None;
    let _: Option<Box<dyn ModuleRuntime>> = None;
    let _: Option<ModuleBindings> = None;
}
