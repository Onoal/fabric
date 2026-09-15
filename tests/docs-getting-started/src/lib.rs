// Keep the tutorial's normal import form in the compile fixture.
#[allow(unused_imports)]
use fabric::prelude::*;

#[derive(Clone)]
pub struct GreetInput {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GreetOutput {
    pub message: String,
}

fabric::component! {
    pub Greeter {
        id: "example.greeter";
        config {}

        operations {
            greet {
                id: "example.greeter.greet";
                input: GreetInput = "example.greeter.greet.input";
                output: GreetOutput = "example.greeter.greet.output";
                handler |input: GreetInput| async move {
                    Ok(GreetOutput {
                        message: format!("hello, {}", input.name),
                    })
                };
            }
        }
    }
}

#[test]
fn getting_started_flow_builds_inspects_materializes_and_invokes() {
    let _component_id = <Greeter as ComponentDefinition>::component_id();
    let built = Fabric::new("example.greeter")
        .expect("valid composition")
        .component(Greeter::define(GreeterConfig {}))
        .build()
        .expect("build");

    let manifest = built.manifest();
    assert_eq!(manifest.components().len(), 1);

    let mut instance = built
        .materialize_named("example.greeter.local")
        .expect("materialize");
    instance.start().expect("start");

    let components = instance.components().expect("component host");
    components
        .materialize::<Greeter>()
        .expect("materialize component");

    let output = futures::executor::block_on(components.invoke_external(
        &greeter::operations::greet(),
        GreetInput {
            name: "Ada".to_owned(),
        },
    ))
    .expect("runtime invocation");
    assert_eq!(output.message, "hello, Ada");

    components
        .dematerialize::<Greeter>()
        .expect("dematerialize component");
    instance.stop();
}
