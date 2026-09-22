# fabric

`fabric` is the main Rust crate for Fabric. This directory implements the
high-level Rust authoring surface over the same public typed machinery
available to handwritten third-party extensions.

Use `fabric::*` for normal Resource, System, Adapter, Component, Host, Fabric,
Instance, and semantic Manifest work. `fabric::prelude::*` remains an optional
compatibility convenience. Use explicit named modules such as
`fabric::authoring`, `fabric::core`, and `fabric::component` for advanced/raw
capability.

Add the SDK package to an application:

```toml
[dependencies]
fabric = { package = "onoal-fabric", version = "0.4.9" }
```

Normal code imports the SDK through its public Rust crate name:

```rust
use fabric::*;
```

For the normal learning path, read [Getting Started](../docs/getting-started.md),
then the [Concepts overview](../docs/concepts/README.md). This document is
package-level API documentation; [Architecture](../docs/architecture.md) and
the [Advanced Raw API](../docs/advanced/raw-api.md) provide deeper context.

## Normal flow

Normal authoring is define, select, compose, build, inspect, materialize,
start, operate Components, then stop:

```rust
use fabric::*;

let built = Fabric::new("example")
    .expect("valid composition id")
    .component(Greeter::define(GreeterConfig {}))
    .build()
    .expect("build");

let manifest = built.manifest();
assert_eq!(manifest.components().len(), 1);

let mut instance = built.materialize_named("example.local").expect("instance");
instance.start().expect("start");
let components = instance.components().expect("component host");
components.materialize::<Greeter>().expect("materialize component");
let output = futures::executor::block_on(
    components.invoke_external(&greeter::operations::greet(), GreeterInput { name: "Ada".into() }),
).expect("runtime invocation");
assert_eq!(output.message, "hello, Ada");
components.dematerialize::<Greeter>().expect("dematerialize component");
instance.stop().expect("stop instance");
```

`BuiltFabric` materializes to `FabricInstance`; raw
`built.composition().materialize(...)` remains available to advanced Core
users. `FabricInstance::components()` is `None` when the Composition has no
Component host. It is a bounded Component operational façade, not a general
Instance service locator.

Host-constrained Adapter compositions materialize explicitly with a
`HostDescriptor` through the host-aware high-level materialization method.

## Stateful runtime authoring

Configuration is immutable declarative input; it is not live runtime state.
`RuntimeState`, `RuntimeContext`, `StatefulRuntimeAuthoring`, and
`StatefulAdapterDefinition` are normal SDK machinery for handwritten authors.
The `resource!`, `system!`, and `adapter!` macros also accept optional `state`
and `lifecycle` sections. State is fresh for each materialization, while
contract handles and lifecycle hooks for that occurrence share it. Hooks are
optional: simple stateless definitions retain successful initialize/start/stop
defaults and healthy reporting. `stop` is deterministic cleanup and may return
an observable error.

## Resource, System, and Adapter authoring

`resource!` defines occurrence-based capability; normal occurrences are
`ResourceId + ResourceName`. `system!` defines instance-wide shared
infrastructure; normal typed Composition has one coherent occurrence per
`SystemId`. Both macros and handwritten implementations use public definition,
selection, contract, and realization seams.

For normal adaptable Resources and Systems, the target semantic API is the
realization contract. Adapter authors name the target once:

```rust
fabric::adapter! {
    ExampleAdapter for ExampleResource {
        config { value: u64; }
        runtime {
            fn current_value(&self) -> ExampleValue {
                ExampleValue::new(self.config().value)
            }
        }
    }
}
```

The target determines whether it is a Resource or System and supplies the
generated API machinery internally through its Rust-resolved type. Normal
Adapter source names no generated
interface and no `for resource` / `for system` discriminator. Adapter config
stays normal typed Rust, and Adapter selection stays Composition truth. The
legacy explicit realization form remains available for a semantic owner that
intentionally uses a different lower-level contract.

## Component dependencies and operations

`component!` declares Resource requirements in `requires {}` and System
requirements in `system {}`. Handlers receive resolved semantic contracts in a
generated dependencies value; a Component never depends on an Adapter.

```rust
fabric::component! {
    Notes {
        id: "example.notes";
        config { prefix: String; }
        requires { storage: ExampleStore(provisional); }
        system { operations: ExampleOperations(version = "^1"); }
        operations {
            save {
                id: "example.notes.save";
                input: SaveInput = "example.notes.save.input";
                output: SaveOutput = "example.notes.save.output";
                context: invocation;
                handler |context, dependencies, input: SaveInput| async move {
                    let _ = (&config.prefix, &context, &dependencies.storage, input);
                    Ok(SaveOutput {})
                };
            }
        }
    }
}
```

Context is opt-in. Handler shapes are `|input|`, `|dependencies, input|`,
`|context, input|`, and `|context, dependencies, input|`. `InvocationContext`
is runtime-supplied provenance (InstanceId, InstanceGeneration, InvocationId,
and root InvocationOrigin), not identity, authorization, tracing, or network
metadata.

A Resource field is a Component-local role. Two same-target requirements have
different local names and can be selected independently:

```rust
requires {
    storage: ExampleStore(provisional);
    cache: ExampleStore(provisional);
}
// Composition selects `Notes::requirements::storage()` and
// `Notes::requirements::cache()` independently.
```

The local role is not the selected ResourceName. The Component declares a need;
Composition binds it to an occurrence.

An operation has one typed output. Domain failure belongs in that output;
`ComponentError` remains the outer Fabric runtime/control plane:

```rust
output: Result<Document, DocumentError> = "example.documents.open.outcome";
handler |input: OpenDocument| async move {
    Ok(repository.open(input.id))
};

// Result<Result<Document, DocumentError>, ComponentError>
let domain_outcome = components.invoke_external(&documents::operations::open(), input).await?;
match domain_outcome {
    Ok(document) => { /* success */ }
    Err(DocumentError::NotFound) => { /* semantic failure */ }
}
```

## Inspection

`FabricManifest` is immutable semantic Composition inspection. It exposes
Resources, Systems, Components, Component Resource bindings, and Component
System bindings. A Resource binding directly identifies the selected
`ResourceId + ResourceName`; normal inspection need not use ModuleId.

```rust
for binding in built.manifest().component_resource_bindings() {
    println!("{} -> {} {:?}", binding.requirement_name(), binding.resource_id(), binding.resource_name());
}
```

Raw backing diagnostics remain intentionally available, but outside normal
semantic inspection:

```rust
let diagnostics = built.manifest().diagnostics();
let _modules = diagnostics.module_declarations();
let _providers = diagnostics.provider_selections();
```

Manifest is not a deployment, package, serialization, reconstruction, or live
runtime-state format.

## Handwritten extensions and raw APIs

Macros are optional. Public handwritten seams include `ResourceDefinition`,
`SystemDefinition`, `AdapterDefinition`, `ComponentDefinition`,
`PrimaryResourceContract`, `PrimarySystemContract`, `Requires`,
`SystemRequires`, and typed realization interfaces. They are available through
the root/prelude and the explicit `authoring` module according to the authoring
need; no generated private module is required.

`FabricBuilder`, `BlockAuthor`, and `CompositionExt` are advanced APIs and are
intentionally imported explicitly:

```rust
use fabric::authoring::FabricBuilder;
```

Raw Modules, Bindings, Contract provider selections, and Component rails are
likewise available through named modules, for example `fabric::core` and
`fabric::component`. They remain useful advanced capability, but normal
packages should not need them.

`fabric::experimental::projection` is shipped for explicit experimentation. It
is not canonical Core ontology, is never prelude-imported, and may change or be
removed in a future minor release while remaining compatibility-sensitive in a
patch line. Binding and Resource Registry research remain
repository-only. Fabric does not define Component-to-Component declarative
dependencies, global registries, scheduler/placement, deployment/reconstruction
formats, dynamic plugins, or IDL generation.
