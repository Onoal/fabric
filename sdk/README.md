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
fabric = { package = "onoal-fabric", version = "0.6.5" }
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

let composition = Fabric::new("example")
    .expect("valid composition id")
    .component(Greeter::define())
    .build()
    .expect("build");

let manifest = composition.manifest();
assert_eq!(manifest.components().len(), 1);

let mut instance = composition.materialize_named("example.local").expect("instance");
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

`Composition` materializes to `FabricInstance`; raw
`composition.core().materialize(...)` remains available to advanced Core
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

## Reusable contributions

The authoring layers have distinct jobs:

| Layer | Meaning |
| --- | --- |
| Definition | what one semantic thing means |
| Contribution | reusable organization of authoring |
| Fabric | complete authoring accumulator |
| Composition | built resolved semantic system |

`Contribution != Composition`, `Contribution != Component`, and
`Contribution != Package`. A contribution has no identity, lifecycle, runtime,
semantic occurrence, or inspection entry. It disappears into `Fabric` before
the single `build()` boundary.

Small systems can stay inline:

```rust
let composition = Fabric::new("hello")?
    .resource(Store::select("main")?.using(MemoryStore::new())?)
    .component(App::define())
    .build()?;
```

Large systems can factor ordinary Rust authoring functions:

```rust
fn storage(name: &'static str) -> impl IntoFabricContribution {
    let store = Store::select(name)
        .expect("store")
        .using(MemoryStore::new())
        .expect("adapter");

    FabricContribution::new()
        .resource(store)
}

fn platform() -> impl IntoFabricContribution {
    FabricContribution::new()
        .with(networking())
        .with(storage("main"))
}

let composition = Fabric::new("aether")?
    .with(platform())
    .with(agent())
    .build()?;
```

Contribution configuration is just Rust input to the function. Conflicts and
provider resolution remain whole-Composition laws checked by `Fabric::build()`;
contribution boundaries do not isolate Resources, Systems, Components,
relations, augmentations, Adapter realizations, raw Blocks, or provider
selections.

## Composition inspection

`Composition` is immutable semantic declarative truth. Normal inspection reads
it directly before materialization: Resources, Systems, Components, realization
mode, Adapter definition identity, declared Host requirements, semantic API
endpoint names, augmentation support, initial Component participation intent,
and resolved semantic Relations are all available without decoding Core
ModuleIds.

```rust
for resource in composition.resources() {
    let realization = resource.realization();
    let host = realization.host_requirement();
    let required_by = resource.required_by().count();
    let endpoints = resource.api().endpoints();
    let _ = (host, required_by, endpoints);
}

for relation in composition.relations() {
    let _owner = relation.owner();
    let _role = relation.role();
    let _target = relation.resolved_target();
}

for component in composition.components() {
    let _initial = component.initial_participation();
}
```

`FabricManifest` remains the Composition-owned backing truth and advanced
manifest view. Legacy authored Component selection entries remain available
for compatibility, but canonical resolved semantic relation truth is
`Composition::relations()`.

Raw Core diagnostics remain intentionally available, but outside normal
semantic inspection:

```rust
let diagnostics = composition.manifest().diagnostics();
let _modules = diagnostics.module_declarations();
let _providers = diagnostics.provider_selections();
```

Manifest is not a deployment, package, serialization, reconstruction, or live
runtime-state format.

## Composition to Instance intent

Component initial participation intent belongs to Composition truth. A
self-realized or Adapter-realized Component defaults to
`ComponentDesiredState::Enabled`; a declaration-only Component defaults to
`ComponentDesiredState::Disabled`. Realized Components can opt out with
`initially_disabled()`:

```rust
let composition = Fabric::new("workers")?
    .component(Worker::define().using(WorkerAdapter::new())?.initially_disabled())
    .build()?;
```

During materialization, each fresh Instance receives its own live desired
controls seeded from that immutable Composition intent. Materialization does
not create observed Component participation, and `start()` does not secretly
reconcile every Component. Desired state and observed participation may differ
until reconstruction or explicit operator control acts.

Initial intent is not a `ComponentControlSnapshot`: snapshots are
Instance-specific live/reconstruction state and keep their existing precedence
in advanced Component-host reconstruction flows. Runtime control mutation never
changes the Composition, and rematerializing the same Composition creates a
fresh generation seeded from the original intent.

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
