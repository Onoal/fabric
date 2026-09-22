# Resource

A **Resource** is a semantic technical capability that can occur one or more
times in a Composition. `ResourceId` identifies the capability kind and
`ResourceName` identifies an occurrence. Its typed contract is what consumers
use; an adaptable Resource may be realized by an Adapter.

```text
Resource = semantic technical capability
Resource occurrence = ResourceId + ResourceName
Resource != Adapter
```

A Resource is not a System with a missing name: Resources model independently
named occurrences, while a System models one coherent shared capability. It is
not a Component either: Components own behavior and consume capabilities. It is
not an Adapter: an Adapter owns one concrete implementation of the semantic
Resource. See [System](system.md), [Component](component.md), and
[Adapter](adapter.md).

Components ask for Resource semantics rather than databases, cloud services,
libraries, or local implementations. Composition chooses the occurrence and
realization that satisfy the requirement.

## Type identity and occurrence identity

`KeyValueStore("primary")` and `KeyValueStore("cache")` can share a
`ResourceId` while having different `ResourceName`, Config, realization, and
Component requirement roles. Resource types are not singletons.

Component-local names are separate from Resource names:

```text
Component role: storage
Composition binding: storage -> KeyValueStore("primary")
```

The role describes meaning to the consumer. `ResourceName` identifies the
occurrence. See [Component](component.md) and [Composition](composition.md).

## Definition, selection, Config, and contract

For normal code, `resource!` declares a Resource kind with Config, identity,
version semantics, Relations, API, and an optional self-realization. The
advanced `ResourceDefinition` trait is the typed lower-level representation of
that same declaration. Neither is an occurrence or a concrete implementation.

`ResourceSelection<R>` is a configured named occurrence:

```text
ResourceDefinition + ResourceName + Config = ResourceSelection
```

Selection is declarative authoring, not live runtime state. Resource Config is
semantic occurrence configuration—not Component, System, Adapter, or Host
configuration.

Resources expose a typed semantic capability through `PrimaryResourceContract`.
Consumers receive that contract, not Adapter internals, `ModuleRuntime`, or a
provider implementation type.

```rust
fabric::resource! {
    pub NoteStore {
        id: "example.note-store";
        version: "0.1.0";
        config { label: String; }
        api {
            fn label(&self) -> String;
        }
    }
}

let primary = NoteStore::select(
    "primary", NoteStoreConfig { label: "primary".to_owned() },
)?;
let built = Fabric::new("example.resources")?.resource(primary).build()?;
```

`resource!` generates ergonomic public machinery, but it is authoring
convenience rather than the ontology. Handwritten `ResourceDefinition` remains
public.

### Self-realization, runtime state, and lifecycle authoring

A Resource is a semantic capability; it does not inherently own executable
machinery. A normal adapted Resource can declare Config, Relations, and API
without `state`, `runtime`, or `lifecycle`. It becomes operational only when a
compatible realization is selected for materialization.

`runtime` explicitly means that this Resource itself owns a live
self-realization or semantic mediation layer. Only then may `state` and
`lifecycle` appear. State is fresh for each materialized live occurrence; it
is not a shared `Arc<Mutex<_>>` hidden in Config. Typed service handles for
that occurrence share the same live state.

```rust
fabric::resource! {
    LocalCounter {
        id: "example.local-counter";
        api {
            fn current(&self) -> usize;
        }
        state { CounterState = CounterState::default(); }
        runtime { fn current(&self) -> usize { self.state.get().current() } }
        lifecycle {
            initialize { Ok(()) }
            stop { self.state.get().close(); Ok(()) }
            health: Health::Healthy;
        }
    }
}
```

This `LocalCounter` form is a **self-realizing Resource**, not normal Resource
anatomy. `initialize` runs after context and dependency binding, `start`
activates its live owner, and `stop` is that owner's fallible cleanup hook.
Fabric calls `stop` even when a materialized generation is abandoned before
start. `RuntimeContext` is bound before normal initialization but may be
absent during early cleanup. Health is an observation separate from Instance
lifecycle: a running live realization may be Healthy, Degraded, or Unavailable
without changing its lifecycle state. Adapter realizations use the same Fabric
lifecycle ownership model; their canonical authoring is documented separately.

The normal Adapter implements the Resource API directly. When a Resource
deliberately needs semantic mediation, it can own only that mediation while an
Adapter supplies an effective lower realization contract. Fabric composes typed
providers rather than selecting one provider per method. See
[Realization](realization.md).

## Version compatibility and realization

### Normal and advanced compatibility authoring

Normal authors declare a semantic `version: "...";` only when the definition
is deliberately versioned. Omitting it means provisional semantics; the
canonical API inherits that enclosing version and its contract identity is
derived from the Resource identity.
No Config declaration means `NoteStore::select("primary")` with no generated
empty Config value. Inline `config { ... }` generates a public typed Config;
`config: MyConfig;` uses a creator-owned Rust type directly.

Advanced compatibility controls are available when an implementation must span
intentional target-version ranges; they are not part of normal package code.

The internal `ResourceSchemaDescriptor` still carries the identity/version that
Fabric uses for compatibility. An Adapter with no explicit support declaration
derives exact support for its compiled target definition. An implementation
that intentionally spans target versions can opt into advanced explicit
compatibility with `supports: "^0.4";`.

`ResourceSchemaDescriptor` combines semantic schema identity and version.
`ResourceId` says what capability exists; schema says which semantic shape and
version it has. This is not crate/package version. `Provisional` compatibility
matches provisional-to-provisional only; `Versioned` schemas use semantic
version requirements. A definition's `resource_id()` must agree with its schema
Resource identity.

Schema compatibility establishes that a realization can bind or materialize;
it does not establish safe replacement, migration, or provider-state transfer.

A self-realizing Resource may materialize directly. For the normal adapted
form, its semantic `api { ... }` is the Adapter requirement and the selected
Adapter owns the concrete live machinery. A differential
[realization boundary](adapter.md) is available for intentional semantic
mediation with a different lower-level interface.

```text
ResourceSelection + compatible Adapter = ResourceRealization
```

```rust
let realized = resource_selection.using(adapter)?;
```

`.using(adapter)` is declarative: it does not start an [Adapter](adapter.md), connect to a
service, or mutate an Instance. It records the occurrence, Adapter provider,
and Adapter provider declaration for later Composition materialization. In the
normal form, a compatible Adapter targets the Resource, supports its schema,
and implements the Resource API directly. Not every Resource requires an
Adapter.

```text
Resource                         Adapter
semantic capability              implementation/realization
named occurrence + semantic Config implementation Config/schema support
typed consumer contract          realization implementation/Host requirements
```

Different occurrences can use different adapters while remaining the same
Resource kind; they do not become “Memory Resources” or “Filesystem Resources”.

## Composition and runtime

`Fabric::resource(...)` accepts a `ResourceSelection` or `ResourceRealization`.
The latter also contributes an Adapter provider and declarative selection.
`FabricManifest` retains Resource identity, name, schema, and semantic bindings.

```text
ResourceDefinition -> ResourceSelection -> optional self/Adapter realization
                   -> Composition -> materialized Instance -> typed contract
```

Resolution is Core declaration binding, not runtime service discovery:

```text
Component requires Resource contract
-> Composition selects occurrence
-> selected Adapter may provide the semantic API directly
-> Core resolves/binds
-> Component receives typed contract
```

## External augmentation

An external semantic may attach to one selected Resource occurrence without
changing `ResourceDefinition`, its schema, or its primary contract. The target
is the actual `ResourceId + ResourceName` occurrence: `Store("primary") + X`
does not make `Store("secondary") + X` true.

```rust
let attachment = ResourceAugmentation::<NoteStore, Readback>::attach(&primary, ())?;
let supported = attachment.using(ReadbackSupport);
let readback = supported.require_from(&primary)?;

let built = Fabric::new("example.resource-augmentation")?
    .resource(primary)
    .resource_augmentation(supported)
    .build()?;
# let _ = (readback, built);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`ResourceAugmentationDefinition` owns the typed semantic contract and Config
for `X`; `ResourceAugmentationSupportDefinition` independently supplies its
implementation. `attach(...)` records bare semantic truth. Only `.using(...)`
adds a provider for `X`, and `require_from(...)` creates a target-bound typed
requirement rather than a free-floating contract lookup. Support may own its
own dependencies, additional implementation contracts, and Host requirement.
See [Augmentation](augmentation.md) for the shared law and Manifest boundary.

## Low-level identity nuance

`fabric-resource` also exposes `ResourceContext`, `ResourceBoundaryId`,
`ResourceScope`, and `ResourceInstanceId`. The latter derives from
`ResourceId + ResourceContext + ResourceName`. This is lower-level context
machinery. Normal high-level Composition occurrence identity remains
`ResourceId + ResourceName`; `Fabric::resource()` needs no `ResourceContext`.
`ResourceInstanceId` is not Fabric `InstanceId`, and a Resource occurrence is
not a live Fabric Instance.

## Boundaries

```text
Resource = named technical capability occurrence
System = shared capability with a different occurrence law
Component = behavior that may require a Resource
Instance = live materialization of the whole Composition
```

Resource is not specifically a database, queue, filesystem, secret, or cache;
those are possible capability semantics. Host requirements belong to Adapter/
Host realization concerns. `ResourceError` covers Resource authoring/model
errors; individual typed contracts own their own method outcomes.

Next: [System](system.md), the coherent shared-capability boundary.
Return to the [Concept map](README.md) or [Architecture](../architecture.md).
