# Getting Started with Fabric

This guide builds one small Fabric system: a Greeter Component with a typed
operation. It shows the full path from declaration to a running Instance.

## Install and import

```toml
[dependencies]
fabric = { package = "onoal-fabric", version = "0.4.5" }
futures = "0.3"
```

```rust
use fabric::*;
```

The tutorial uses `futures::executor::block_on` only to drive the asynchronous
operation in a small standalone program. Applications normally use their own
async runtime.

## Define behavior

```rust
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
```

`Greeter` is a declaration of behavior. Defining it does not start a runtime or
perform an operation.

## Compose, inspect, and run

```rust
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
components.materialize::<Greeter>().expect("materialize component");

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
instance.stop().expect("stop");
```

The `Fabric` builder accumulates declarations. `build()` validates them and
produces a reusable Composition artifact plus its semantic Manifest. Reading
the Manifest inspects declaration-time truth; it does not inspect live runtime
state.

`materialize_named()` creates one Instance from that declaration. Starting the
Instance makes its runtime available. The bounded Component façade then
materializes Greeter, invokes its typed operation, and dematerializes it before
the Instance stops.

In short:

```text
definition -> Composition -> build -> Manifest inspection
           -> Instance materialization -> start -> operation -> stop
```

Next, read [Composition](concepts/composition.md) to understand the declaration
that `build()` created. The [Concepts overview](concepts/README.md) maps the
remaining pieces; read [Architecture](architecture.md) for the precise model.
