# Getting started with Fabric

This guide follows one small system from semantic definition to a running
Instance. Its complete source is compiled in `tests/docs-getting-started`.

## 1. Define what the capability means

`Store` is a Resource. It states only its identity and the typed API consumers
receive; it has no fake self runtime.

```rust
use fabric::*;

fabric::resource! {
    pub Store {
        id: "example.getting-started.store";
        api { fn count(&self) -> usize; }
    }
}
```

## 2. Provide a concrete realization

`MemoryStore` says it realizes `Store`. The target supplies the normal
realization contract, so there is no `implements`, generated trait name, or
Resource forwarding method to write.

```rust
fabric::adapter! {
    pub MemoryStore for Store {
        runtime { fn count(&self) -> usize { 7 } }
    }
}
```

The Adapter owns concrete implementation machinery. A stateful Adapter would
put its mutable state and lifecycle hooks here, not in `Store` Config.

## 3. Consume the semantic API

Components require semantic capabilities, never a concrete Adapter. Canonical
Component declaration names those needs through `relations` and declares its
callable behavior through `api`. The current tutorial then uses the
transitional legacy self-realizing frontend solely because canonical Component
runtime authoring is the next slice.

```rust
fabric::component! {
    pub Greeter {
        id: "example.greeter";
        requires { store: Store(provisional); }
        operations {
            greet {
                id: "example.greeter.greet";
                input: GreetInput = "example.greeter.greet.input";
                output: GreetOutput = "example.greeter.greet.output";
                handler |dependencies, input: GreetInput| async move {
                    Ok(GreetOutput {
                        message: format!("hello, {} ({})", input.name, dependencies.store.count()),
                    })
                };
            }
        }
    }
}
```

`GreetInput` and `GreetOutput` are ordinary application Rust types; the
compiled fixture defines them in full.

## 4. Select, compose, materialize, and run

Selection records declarative truth. `build()` validates the Composition;
`materialize_named()` creates one live generation; `start()` makes the graph
operational. Component participation is then explicit.

```rust
let built = Fabric::new("example.greeter")?
    .resource(
        Store::select("primary")?
            .using(MemoryStore::new())?,
    )
    .component(Greeter::define(GreeterConfig {}))
    .build()?;

let mut instance = built.materialize_named_on(
    "example.greeter.local",
    &HostDescriptor::native(),
)?;
instance.start()?;

let components = instance.components().expect("Component host");
components.materialize::<Greeter>()?;
let output = futures::executor::block_on(components.invoke_external(
    &greeter::operations::greet(),
    GreetInput { name: "Ada".to_owned() },
))?;
assert_eq!(output.message, "hello, Ada (7)");

components.dematerialize::<Greeter>()?;
instance.stop()?;
```

```text
definition -> selected realization -> Composition -> Instance generation
                                                 -> start -> typed operation -> stop
```

`Composition` remains reusable declaration-time truth. `Instance` is one live
generation and stops terminally; materialize again for a fresh generation.

Next, read [Resource](concepts/resource.md), [Adapter](concepts/adapter.md),
and [Composition](concepts/composition.md). The [Architecture](architecture.md)
explains how typed requirements, providers, binding, lifecycle, and health fit
beneath this normal path.
