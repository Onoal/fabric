# onoal-fabric

`onoal-fabric` is the umbrella crate and normal entry point for Fabric. It
exports the high-level Rust authoring surface and the normal macros while
keeping Core and generated implementation machinery opt-in.

Use this crate for normal applications and package authors. The sibling crates
exist so low-level integrations can depend on focused primitives; they are not
required to understand or use Fabric. Start with the
[Fabric manual](https://github.com/Onoal/fabric/tree/main/docs) and its
[Getting Started guide](https://github.com/Onoal/fabric/blob/main/docs/getting-started.md).

Use `fabric::*` for normal Resource, System, Adapter, Component, Host, Fabric,
Instance, and semantic Manifest work. `fabric::prelude::*` remains an optional
compatibility convenience. Use explicit named modules such as
`fabric::authoring`, `fabric::core`, and `fabric::component` for advanced/raw
capability.

Add the SDK package to an application:

```toml
[dependencies]
fabric = { package = "onoal-fabric", version = "0.5.3" }
```

Normal code imports the SDK through its public Rust crate name:

```rust
use fabric::*;
```

For the normal learning path, read
[Getting Started](https://github.com/Onoal/fabric/blob/main/docs/getting-started.md),
then the [manual index](https://github.com/Onoal/fabric/tree/main/docs). This
document is package-level API documentation;
[Architecture](https://github.com/Onoal/fabric/blob/main/docs/architecture.md)
and the [Advanced Raw API](https://github.com/Onoal/fabric/blob/main/docs/advanced/raw-api.md)
provide deeper context.

## Normal flow

Normal authoring is define, select, compose, build, inspect, materialize,
start, operate Components, then stop:

```rust
use fabric::*;

let built = Fabric::new("example")
    .expect("valid composition id")
    .component(Greeter::define())
    .build()
    .expect("build");

let manifest = built.manifest();
assert_eq!(manifest.components().len(), 1);

let mut instance = built.materialize_named("example.local").expect("instance");
instance.start().expect("start");
let components = instance.components().expect("component host");
components.materialize::<Greeter>().expect("materialize component");
let output = futures::executor::block_on(
    components.invoke_external(&greeter::api::greet(), GreeterInput { name: "Ada".into() }),
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
`StatefulAdapterDefinition` are explicit `fabric::authoring` machinery for
handwritten advanced authors.
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
stays normal typed Rust, and Adapter selection stays Composition truth. A
differential realization boundary is available when a semantic owner
intentionally uses a different lower-level contract.

## Component declaration and transitional operations

Canonical `component!` declaration uses the same incremental Fabric language
as Resources and Systems: identity, optional Config, optional Relations, and
optional API. It declares a semantic behavioral participant; it does not
create a handler, participation, or self realization.

```rust
fabric::component! {
    Notes {
        id: "example.notes";
        config { prefix: String; }
        relations {
            requires {
                storage: ExampleStore;
                operations: ExampleOperations;
            }
        }
        api { fn save(&self, input: SaveInput) -> SaveOutput; }
    }
}
```

Relation fields are Component-local roles. `storage` and `cache` can name two
distinct requirements for the same target; the target determines whether it is
a Resource or System. Composition supplies matching semantic capabilities when
a realization later needs them.

The old `operations`, handler, top-level `requires`/`system`, and top-level
`teardown` forms were removed in 0.5.4. Use `relations`, `api`, and `runtime`
instead. Invocation provenance remains available only through the named
advanced invocation API; it is not a macro-level semantic API argument.

An API endpoint has one typed output. Domain failure belongs in that output;
`ComponentError` remains the outer Fabric runtime/control plane. For example,
an `api` method returning `Result<Document, DocumentError>` is invoked as
`Result<Result<Document, DocumentError>, ComponentError>`.

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
