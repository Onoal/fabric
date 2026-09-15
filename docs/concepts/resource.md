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

`ResourceDefinition` is typed authoring for a Resource kind: Config type,
`ResourceId`, schema, declaration construction, and optional direct runtime
materialization. It is not an occurrence.

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
        schema: provisional;
        config { label: String; }
        contracts { primary Api {
            id: "example.note-store.api"; version: provisional;
            fn label(&self) -> String;
        }}
        runtime { fn label(&self) -> String { self.config.label.clone() } }
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

## Schema and realization

`ResourceSchemaDescriptor` combines semantic schema identity and version.
`ResourceId` says what capability exists; schema says which semantic shape and
version it has. This is not crate/package version. `Provisional` compatibility
matches provisional-to-provisional only; `Versioned` schemas use semantic
version requirements. A definition's `resource_id()` must agree with its schema
Resource identity.

A Resource may materialize directly, or implement
`AdaptableResourceDefinition` with a typed [RealizationContract](adapter.md).

```text
ResourceSelection + compatible Adapter = ResourceRealization
```

```rust
let realized = resource_selection.using(adapter)?;
```

`.using(adapter)` is declarative: it does not start an [Adapter](adapter.md), connect to a
service, or mutate an Instance. It records the occurrence, Adapter provider,
and provider selection for later Composition materialization. A compatible
Adapter targets the Resource, supports its schema, and implements its explicit
realization contract. Not every Resource requires an Adapter.

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
ResourceDefinition -> ResourceSelection -> optional ResourceRealization
                   -> Composition -> materialized Instance -> typed contract
```

Resolution is Core declaration binding, not runtime service discovery:

```text
Component requires Resource contract
-> Composition selects occurrence
-> realization may use Adapter
-> Core resolves/binds
-> Component receives typed contract
```

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
