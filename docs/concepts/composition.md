# Composition

A **Composition** is Fabric's declarative assembly of participating
definitions, requirements, provider choices, and structural relationships
before anything is running.

It answers: *what belongs to this system, and how do its declared needs fit
together?* It does not answer what is currently running. That distinction is
fundamental:

```text
Composition = the declaration of what is assembled
Instance    = one live materialization of that declaration
```

One Composition can therefore be inspected, reused, and materialized into
multiple independent Instances.

```text
Definitions and declarations
            |
            v
       Composition
       |- participants
       |- requirements
       |- provider selections
       `- structural relationships
            |
            | materialize
            v
         Instance
```

Materialization also accepts occurrence-specific intent:

```text
Composition
    + MaterializationProfile
    + Host
        ↓
    MaterializationPlan
        ↓
    Instance
```

`MaterializationProfile` is not Composition truth. It is supplied when creating
an effective `MaterializationPlan` and retained as materialization provenance.
A Composition can therefore produce a `default` Plan and a `diagnostic` Plan
without changing its declared participants, relations, manifest, or semantic
inspection.

`MaterializationPlan` is frozen, non-live materialization truth prepared from
one Composition under one Profile and Host context. It can be inspected before
an Instance exists. It does not have lifecycle, health, runtime state,
Component participation, or an `InstanceGeneration`.

## Why Composition exists

Without a declaration boundary, behavior tends to couple directly to concrete
implementations, construction order, runtime objects, and provider-specific
choices. Composition gives Fabric one place to declare that participants
exist, that requirements exist, and that compatible providers have been
chosen—before a live runtime is created.

This is not a generic service registry or a deployment description. It is the
declarative truth Core validates and later materializes.

## Fabric and Composition

Normal Rust authoring begins with [`Fabric`](../../sdk/README.md), the
high-level authoring surface. `Fabric` accumulates typed semantic
contributions; it is neither runtime state nor the resulting Composition.

Calling `build()` validates those contributions and produces the normal SDK
`Composition`:

```text
Fabric                    = normal high-level authoring
Composition               = complete immutable declared semantic truth
Composition.manifest()    = immutable semantic/provenance inspection
Composition.core()        = deliberate advanced Core escape hatch
fabric::core::Composition = generic resolved structural representation
```

The SDK Composition wraps the resolved Core Composition; neither replaces the
other. Normal users materialize the SDK Composition. Advanced users may inspect
the Core value through `Composition::core()`.

### Example A: a minimal Composition

This complete example builds a declaration only. It does not create or start
an Instance.

```rust
use fabric::*;

let composition = Fabric::new("example.minimal")
    .expect("valid CompositionId")
    .build()
    .expect("valid declaration");

assert_eq!(composition.id().as_str(), "example.minimal");
assert!(composition.manifest().resources().is_empty());
```

The identifier above is a `CompositionId`: it identifies the declaration, not
an `InstanceId` or an Instance generation.

`CompositionId` is caller-selected declaration identity, not a content hash or
revision sequence. Two separately authored Compositions may reuse the same ID
while containing different declarations. Fabric does not infer content
equality, version ordering, or supersession from that reuse.

## What a normal Composition contains

Normal Fabric authoring can contribute Components, Resource occurrences,
Systems, Adapter-backed realizations, semantic requirements, provider
selections, and the structural declarations needed to connect them. A
Component-bearing high-level Composition may also declare one bounded
operational export so its Component capability can be used through the
Instance API.

These are semantic facts, not a promise that Composition stores a flat enum of
every concept. High-level authoring lowers through Core module and contract
declarations. Use `FabricManifest` to inspect the high-level semantic picture;
use `manifest.diagnostics()` only when advanced Core backing details are
needed.

### Declaration time and runtime

| Declaration time — Composition | Effective materialization — Plan | Runtime — Instance |
| --- | --- | --- |
| Which participants should exist? | Which participants are prepared for this occurrence context? | Which runtime objects were materialized? |
| What does a Component require? | Which existing v1 realization truth applies? | What lifecycle state is live? |
| Which provider satisfies a requirement? | Which Host context was validated? | Which Instance generation is running? |
| Which compatibility requirements apply? | Which Profile applies? | Which operations are executing? |
| What initial Component participation was declared? | What initial Component intent will seed a fresh Instance? | What Component participation is currently desired and observed? |
| No occurrence Profile is stored here. | No live generation is stored here. | Which Plan/Profile produced this Instance? |

The Plan and Instance columns are deliberately not Composition state.

## Requirements and provider selections

A Component declares semantic needs such as a required Resource or System.
Those requirements participate in the Composition; the Component does not
select a concrete Adapter or discover a runtime object for itself.

**Compatibility is not selection.** Compatibility says a provider *can*
satisfy a requirement. Selection says this Composition *chooses* one compatible
provider for that requirement. If more than one provider is compatible, a
selection removes ambiguity as declarative truth before materialization.

### Example B: semantic dependency declarations

The following complete definitions declare a Resource and a Component with two
named Resource requirements. The requirement names express the Component's
local roles; they are not Resource occurrence names.

```rust
use fabric::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreObservation {
    pub primary: String,
    pub cache: String,
}

fabric::resource! {
    pub NoteStore {
        id: "example.note-store";
        version: "0.1.0";
        config { label: String; }
        api {
            fn label(&self) -> String;
        }
        runtime {
            fn label(&self) -> String { self.config().label.clone() }
        }
    }
}

fabric::component! {
    pub StoreProbe {
        id: "example.store-probe";
        relations {
            requires {
                primary_store: NoteStore(version = "^0.1");
                cache_store: NoteStore(version = "^0.1");
            }
        }
        api { fn inspect(&self) -> StoreObservation; }
        runtime {
            fn inspect(&self) -> StoreObservation {
                StoreObservation {
                    primary: self.relations().primary_store.label(),
                    cache: self.relations().cache_store.label(),
                }
            }
        }
    }
}
```

`primary_store` and `cache_store` are Component-local requirement roles. They
can bind independently even though both target `NoteStore`.

### Example C: select Resource occurrences explicitly

Resource type and Resource occurrence are different facts. The two selections
below have the same `ResourceId` but distinct names: `"primary"` and
`"cache"`. The Composition binds each Component role to the intended
occurrence.

```rust
let primary = NoteStore::select(
    "primary",
    NoteStoreConfig { label: "primary".to_owned() },
)
.expect("valid ResourceName");
let cache = NoteStore::select(
    "cache",
    NoteStoreConfig { label: "cache".to_owned() },
)
.expect("valid ResourceName");

let composition = Fabric::new("example.store-composition")
    .expect("valid CompositionId")
    .resource(primary.clone())
    .resource(cache.clone())
    .component(
        StoreProbe::define(),
    )
    .build()
    .expect("unambiguous selected providers");

let bindings = composition.manifest().component_resource_bindings();
assert_eq!(bindings[0].requirement_name().as_str(), "primary_store");
assert_eq!(bindings[1].requirement_name().as_str(), "cache_store");
```

The same law applies to Component-to-[System](system.md) requirements, but
Systems are a different semantic world: normal typed authoring has one
coherent System occurrence per `SystemId` in a Composition, rather than
Resource-style named occurrences.

An [Adapter](adapter.md), where an adaptable Component, Resource, or System
needs one, realizes that semantic target through its public realization
interface. A Component still requires semantic Resources or Systems rather
than concrete Adapters; an Adapter-realized Component uses an Adapter only for
its own realization. The dedicated Adapter page covers realization authoring
in depth.

### Two common mistakes

```text
Wrong:   a Component chooses a concrete Adapter directly.
Correct: the Component declares a Resource/System requirement; the
         Composition selects a compatible provider.

Wrong:   build() means the runtime has started.
Correct: build() creates validated declaration and inspection truth;
         materialization creates live runtime state.
```

## Build, then materialize

`build()` is validation and declarative resolution, not startup. Core snapshots
module declarations, validates structural identities, requirements and
providers, freezes selected-provider bindings and dependency order, and assigns
each declared export to one compatible declared provider. An invalid
declarative system therefore fails before it becomes an Instance.

Materialization does not choose providers again. It validates the target Host,
constructs fresh runtimes, and verifies that each already-selected provider
produces the live contracts promised by its frozen declaration. A live runtime
can still fail that verification, but it cannot change Composition ownership.

Continuing Example C, only after a successful build can the Composition be
materialized:

```rust
let mut instance = composition
    .materialize_named("example.store-composition.local")
    .expect("materialize a separate live Instance");
instance.start().expect("start");
// Materialize and invoke declared Components through instance.components().
instance.stop().expect("stop");
```

The same built Composition can produce multiple independent Instances. Their
lifecycle, runtime objects, and generations are separate even though their
declaration is shared.

Composition itself never starts or stops. Its materialized Instance coordinates
runtime participants through bind, initialize, start, and fallible `stop`.
Stopping is deterministic cleanup for every materialized participant, including
participants in an abandoned startup; it does not mutate Composition truth.

Host requirements follow the same boundary. A Composition may carry declared
environmental compatibility requirements through its realizations; a concrete
`HostDescriptor` is supplied to `materialize_named_on` when those requirements
must be evaluated. A Host requirement is not a live Host stored in the
Composition.

## Reuse with ordinary Rust

Fabric 0.1 does not need a deployment DSL, serialized Composition format,
Manifest reconstruction format, or global package graph for reuse. An ordinary
Rust function can return a partially assembled `Fabric`, and callers can extend
it before building.

### Example D: reusable authoring

```rust
fn local_store_stack() -> Fabric {
    let primary = NoteStore::select(
        "primary",
        NoteStoreConfig { label: "primary".to_owned() },
    )
    .expect("valid ResourceName");

    Fabric::new("example.reusable")
        .expect("valid CompositionId")
        .resource(primary)
}

let composition = local_store_stack()
    .component(StoreProbe::define(StoreProbeConfig {}))
    .build()
    .expect("one available provider can satisfy both requirements");
```

In a real caller, a Component with requirements must receive providers that
are available and unambiguous, or explicitly selected as in Example C. A
single compatible provider may satisfy more than one requirement; explicit
selection is needed when the roles must use different occurrences.

## Advanced structural machinery

The normal path is `Fabric::new(...).resource(...).system(...).component(...)
.build()`. Advanced users can work directly with `CompositionBuilder`, `Block`,
`ContractProviderSelection`, and `CompositionExport` through the named raw API.

`Block` is grouping and reporting machinery, not semantic dependency topology.
Core resolves provider-before-consumer ordering from the declaration graph. For
otherwise independent modules, its current deterministic tie-break follows the
flattened Block/module insertion order; that order is not semantic dependency
priority. See the [Advanced Raw API](../advanced/raw-api.md) for that level of
authoring.

## Composition and Manifest

Composition is the declarative Core truth used for validation and
materialization. `FabricManifest` is immutable, read-only semantic inspection
of the high-level authored Composition. A Manifest is not the Composition
itself, a deployment format, a reconstruction format, or runtime observation
state.

Composition also does not own live lifecycle state, Instance generation,
running operations, mutable runtime service discovery, scheduler or placement
policy, capacity, a global registry, identity or authority, or deployment
serialization.

External augmentation attachments are also declarative Composition truth. A
Resource, System, or Component attachment records additional semantic meaning
for its existing target; an optional support provider is ordinary backing
provider truth resolved through Core selections. `FabricManifest` exposes
explicit semantic augmentation entries, while `manifest.diagnostics()` exposes
the lower-level Modules and provider selections used to implement them. The
semantic entry is not a provider identity, support dependency list, Host
requirement, or runtime record. See [Augmentation](augmentation.md).

Next: [Instance](instance.md), the live materialization boundary. You can also
return to the [Concept map](README.md).
