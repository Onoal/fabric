# System

A **System** is Fabric's semantic model for one coherent shared capability
identified by a `SystemId` within a Composition. Components and other Systems
may require that capability; Composition decides how it is realized.

```text
System = one coherent shared semantic capability in a Composition
System != Resource without a name
System != Adapter
```

A System is not a Resource without a name: it represents one coherent shared
capability in a Composition. It is not a Component, which owns behavior and
consumes capabilities, and it is not an Adapter, which owns a concrete
implementation. See [Resource](resource.md), [Component](component.md), and
[Adapter](adapter.md).

## Why System exists

Resources naturally represent independently named technical occurrences, such
as `Store("primary")` and `Store("cache")`. A System represents a capability
that is shared and coherent for the assembled declaration, such as a clock
domain or an operation-coordination capability. The distinction is semantic,
not a statement about infrastructure size.

| | Resource | System |
| --- | --- | --- |
| Meaning | named technical capability occurrence | coherent shared capability |
| Identity | `ResourceId + ResourceName` | `SystemId` |
| Same-kind occurrences | multiple named occurrences | one in normal typed Composition authoring |
| Config | per named occurrence | per coherent System selection |
| Component can require it | yes | yes |
| Adapter realization | optional | optional |

`SystemId` is the stable identity of the capability, for example
`example.operations`. Normal authoring has no `SystemName` or
`SystemInstanceId`: a System is not a Resource whose name was omitted.

## Definition and selection

For normal code, `system!` declares a System kind with Config, identity,
version semantics, Relations, API, and an optional self-realization. The
advanced `SystemDefinition` trait is the typed lower-level representation of
that same declaration. Neither is a configured contribution or a concrete
implementation.

`SystemSelection<S>` is that one configured contribution:

```text
SystemDefinition + Config = SystemSelection
```

Unlike `ResourceDefinition::select`, `SystemDefinition::select` accepts no
name. The normal module identity is derived from `SystemId` alone, which makes
two independently configured selections of the same `SystemId` invalid in one
Composition. The build fails before materialization rather than creating two
coherent Systems with conflicting configurations.

System Config belongs to this semantic System selection. It is not Component,
Resource, Adapter, or Host Config. A `PrimarySystemContract` is the typed
semantic capability consumers receive—not an Adapter, provider Module, or
global manager.

### Normal authoring

`system!` is the normal ergonomic frontend. It generates public typed
authoring machinery (definition, Config, identity, schema, semantic API,
and runtime implementation where declared), but the macro is not the System
ontology. Handwritten `SystemDefinition` is public too.

```rust
fabric::system! {
    pub Operations {
        id: "example.operations";
        version: "0.1.0";
        config { seed: u64; }
        api {
            fn marker(&self) -> u64;
        }
        runtime { fn marker(&self) -> u64 { self.config().seed } }
    }
}

let operations = Operations::select(OperationsConfig { seed: 7 })?;
let built = Fabric::new("example.system")?.system(operations).build()?;
```

`Fabric::system(...)` adds semantic System declaration truth and, only when
authored or selected, its live realization participant. The
`SystemManifestEntry` contains `SystemId` and schema; there is no
ResourceName-equivalent in that Manifest entry.

### Self-realization, runtime state, and lifecycle authoring

A System is a semantic shared capability; Config, Relations, and API do not
make it a concrete live implementation. A normal adapted System may declare
only those semantic facets and requires a compatible realization before it can
materialize.

A System that itself owns executable behavior declares `runtime`; that is an
explicit self-realization or semantic mediation layer. Only that live owner
may declare `state` and `lifecycle`. Its state is fresh for each materialized
Instance generation while Config remains declarative selection input. Hooks
use Fabric's common lifecycle spine: `initialize` prepares after normal
binding, `start` activates, `stop` releases occurrence-owned machinery even
on abandoned startup, and `health` reports ability separately from lifecycle.
Adapter realizations use the same ownership model without making backend
health identical to System health.

The normal Adapter implements the System API directly. A System may explicitly
mediate part of its API while an Adapter supplies the remaining effective
realization contract. Consumers still receive only the System API and Core
composes typed providers, not method-level providers. See
[Realization](realization.md).

## Version compatibility and direct realization

### Normal and advanced compatibility authoring

For normal `system!` authoring, `version: "...";` is the semantic version
word. Omit it for provisional semantics; the canonical API inherits the
enclosing System version and has an owner-derived contract identity. No Config declaration means
`System::select()` with no generated empty Config value. Inline `config { ...
}` generates a public typed Config; `config: MyConfig;` uses a creator-owned
Rust type directly.

Advanced compatibility controls are for deliberate cross-version
implementations, not normal package code.

An Adapter defaults to exact compatibility with the System definition named in
its header. Use `supports: "^0.4";` only when an implementation deliberately
supports a wider semantic-version range.

`SystemSchemaDescriptor` combines `SystemSchemaIdentity` and
`SystemSchemaVersion`; `SystemSchemaRequirement` expresses compatibility.
`SystemId` answers *which capability?* Schema answers *which semantic shape
and version?* Neither is a crate version. `Provisional` matches only
`Provisional`; `Versioned` schemas use semantic-version requirements.

Schema compatibility establishes that a realization can bind or materialize;
it does not establish safe replacement, migration, or System-state transfer.

`SystemSelection::new()` verifies that `SystemDefinition::system_id()` equals
the System identity carried by its schema. A schema for another System cannot
silently describe this one.

A self-realizing System may materialize directly. A semantic System with no
self realization remains declaration-only because
`SystemDefinition::materialize()` defaults to `None`; it can still define,
select, and declare semantic truth, but supplies no native local runtime on
its own.

## Adapter realization

For a normal adapted System, its semantic `api { ... }` is the realization
contract. A compatible Adapter targets that System, receives its exact schema
support by default, and provides the typed System API directly. The System
does not acquire a forwarding runtime merely because it is adapted. A
differential [realization boundary](adapter.md) is available for a deliberately
different lower-level implementation interface.

```text
SystemSelection + compatible Adapter = SystemRealization
```

```rust
let realized = system_selection.using(adapter)?;
```

`.using(adapter)` is declarative: it does not
start a System, connect to a service, modify an Instance, or perform discovery.
It records realization truth for later Composition materialization. In the
normal form the Adapter-owned provider occupies the selected System's semantic
API slot. Incompatible schema support returns `SystemCompatibilityError` before
runtime.

One Composition may select a local Adapter and another a remote Adapter for
the same System semantics. That does not redefine the System. In contrast,
normal authoring does not model `MessagingSystem("primary")` and
`MessagingSystem("secondary")`; there is one selected realization for that
`SystemId` in a Composition.

## Dependencies and consumers

A Component requires a System semantic contract; it does not instantiate or
name the System, and it does not choose an Adapter.

```text
Component requires System contract
        -> Composition contributes System
        -> optional Adapter realizes it
        -> Core resolves and binds
        -> Component receives typed contract
```

For example, a Component requirement names `Operations`, not a concrete
`LocalOperationsAdapter`. See [Component](component.md) for handler syntax.
`system!` also supports typed System-to-System dependencies:

```text
DerivedSystem requires BaseSystem
```

Neither participant owns the other. Both remain Composition participants, and
Core resolves the provider before the dependent runtime. This is ordinary
declarative dependency ordering—not a separate System lifecycle manager.

## Composition scope and runtime

The one-System law applies **per normal typed Composition**, not globally. Two
separate Compositions can select the same `SystemId` with different Config or
Adapters. One Composition can also materialize into multiple Fabric Instances;
each materialization has its own live realization unless an implementation
explicitly uses external shared state.

```text
SystemDefinition
      -> select(Config)
SystemSelection
      -> optional self/Adapter realization
      -> Composition
materialized Instance -> live typed System contract
```

| Concept | Declaration/authoring | Runtime |
| --- | --- | --- |
| `SystemId` | semantic identity | identity preserved |
| Config | selection input | used by its realization |
| schema | compatibility truth | not discovery |
| System dependency | declared | resolved and bound |
| Adapter selection | Composition truth | materialized provider |
| semantic API | declared | live typed value |

## External augmentation

An independently owned semantic `X` may attach to a selected System without
changing `SystemDefinition`, its schema, or its primary contract. The target is
the selected System occurrence under its `SystemId`; augmentation does not add
a `SystemName` or permit a second normal typed System occurrence.

```rust
let attachment = SystemAugmentation::<Operations, DriftObservation>::attach(&operations, ())?;
let supported = attachment.using(DriftSupport);
let drift = supported.require_from(&operations)?;

let built = Fabric::new("example.system-augmentation")?
    .system(operations)
    .system_augmentation(supported)
    .build()?;
# let _ = (drift, built);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`SystemAugmentationDefinition` owns only `X`'s typed semantic contract and
Config. `SystemAugmentationSupportDefinition` independently supplies a
provider; support is not part of the base System or an Adapter identity. A
bare attachment is inspectable semantic truth but does not satisfy a consumer
of `X`. `require_from(...)` ties a typed requirement to this selected System
and its support provider. See [Augmentation](augmentation.md).

## Boundaries

```text
System != global singleton, registry, scheduler, control plane, or service locator
System != Component: Component is behavior; System is shared capability
System != Adapter: Adapter is one realization of System semantics
System != Instance: Instance materializes the whole Composition
System != Host: Host compatibility may constrain an Adapter realization
System != distributed system: deployment topology does not define this concept
```

Wrong: *a System is a Resource without `ResourceName`.*

Correct: Resource and System have intentionally different occurrence laws.

Wrong: *a Component requires an Adapter or SystemId is global deployment
identity.*

Correct: Components require System semantics; Composition selects a
realization, and `SystemId` identifies one coherent capability only within its
Composition.

`SystemCompatibilityError` covers System input, identity, and schema
compatibility. Typed contract methods own their own domain outcomes.

Next: [Adapter](adapter.md), the realization boundary.
Return to the [Concept map](README.md) or [Architecture](../architecture.md).
