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

## Why Composition exists

Without a declaration boundary, behavior tends to couple directly to concrete
implementations, construction order, runtime objects, and provider-specific
choices. Composition gives Fabric one place to declare that participants
exist, that requirements exist, and that compatible providers have been
chosen—before a live runtime is created.

This is not a generic service registry or a deployment description. It is the
declarative truth Core validates and later materializes.

## Fabric, Composition, and BuiltFabric

Normal Rust authoring begins with [`Fabric`](../../sdk/README.md), the
high-level authoring surface:

```rust
use fabric::prelude::*;

let fabric = Fabric::new("example.composition")
    .expect("valid CompositionId")
    // .resource(...)
    // .system(...)
    // .component(...)
    ;
```

`Fabric` accumulates typed semantic contributions. It is not the runtime and
is not itself Core's resulting Composition. Calling `build()` validates those
contributions and produces `BuiltFabric`:

```text
Fabric       = normal high-level authoring
Composition  = validated declarative Core truth
BuiltFabric  = Composition + FabricManifest
```

`BuiltFabric::composition()` exposes the Core Composition for advanced uses;
`BuiltFabric::manifest()` exposes semantic inspection for normal users. The
two are related, but neither replaces the other.

### Example A: a minimal Composition

This complete example builds a declaration only. It does not create or start
an Instance.

```rust
use fabric::prelude::*;

let built = Fabric::new("example.minimal")
    .expect("valid CompositionId")
    .build()
    .expect("valid declaration");

assert_eq!(built.composition().id().as_str(), "example.minimal");
assert!(built.manifest().resources().is_empty());
```

The identifier above is a `CompositionId`: it identifies the declaration, not
an `InstanceId` or an Instance generation.

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

| Declaration time — Composition | Runtime — Instance |
| --- | --- |
| Which participants should exist? | Which runtime objects were materialized? |
| What does a Component require? | What lifecycle state is live? |
| Which provider satisfies a requirement? | Which Instance generation is running? |
| Which compatibility requirements apply? | Which operations are executing? |

The right column is deliberately not Composition state.

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
use fabric::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreObservation {
    pub primary: String,
    pub cache: String,
}

fabric::resource! {
    pub NoteStore {
        id: "example.note-store";
        schema: provisional;
        config { label: String; }
        contracts {
            primary Api {
                id: "example.note-store.api";
                version: provisional;
                fn label(&self) -> String;
            }
        }
        runtime {
            fn label(&self) -> String { self.config.label.clone() }
        }
    }
}

fabric::component! {
    pub StoreProbe {
        id: "example.store-probe";
        config {}
        requires {
            primary_store: NoteStore(provisional);
            cache_store: NoteStore(provisional);
        }
        operations {
            inspect {
                id: "example.store-probe.inspect";
                input: () = "example.store-probe.inspect.input";
                output: StoreObservation = "example.store-probe.inspect.output";
                handler |dependencies, _input: ()| async move {
                    Ok(StoreObservation {
                        primary: dependencies.primary_store.label(),
                        cache: dependencies.cache_store.label(),
                    })
                };
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

let built = Fabric::new("example.store-composition")
    .expect("valid CompositionId")
    .resource(primary.clone())
    .resource(cache.clone())
    .component(
        StoreProbe::define(StoreProbeConfig {})
            .select_named_resource_provider(
                &store_probe::requirements::primary_store(),
                &primary,
            )
            .select_named_resource_provider(
                &store_probe::requirements::cache_store(),
                &cache,
            ),
    )
    .build()
    .expect("unambiguous selected providers");

let bindings = built.manifest().component_resource_bindings();
assert_eq!(bindings[0].requirement_name().as_str(), "primary_store");
assert_eq!(bindings[0].resource_name().as_str(), "primary");
assert_eq!(bindings[1].requirement_name().as_str(), "cache_store");
assert_eq!(bindings[1].resource_name().as_str(), "cache");
```

The same law applies to Component-to-System requirements, but Systems are a
different semantic world: normal typed authoring has one coherent System
occurrence per `SystemId` in a Composition, rather than Resource-style named
occurrences.

An Adapter, where a Resource or adaptable System needs one, realizes that
semantic capability through its public realization interface. A Component still
requires the Resource or System, not the Adapter. The dedicated Adapter page
will cover realization authoring in depth.

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

`build()` is validation, not startup. Core validates structural identities,
requirements and providers, selected-provider compatibility, declaration
compatibility, and unique declared export identities. Materialization then
checks that each declared export resolves to one compatible runtime contract.
An invalid declarative system therefore fails before it becomes an Instance.

Continuing Example C, only after a successful build can the Composition be
materialized:

```rust
let mut instance = built
    .materialize_named("example.store-composition.local")
    .expect("materialize a separate live Instance");
instance.start().expect("start");
// Materialize and invoke declared Components through instance.components().
instance.stop();
```

The same built Composition can produce multiple independent Instances. Their
lifecycle, runtime objects, and generations are separate even though their
declaration is shared.

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

let built = local_store_stack()
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
Core resolves binding, initialization, and lifecycle ordering from the
declaration graph—not Block order, authoring order, or insertion order. See
the [Advanced Raw API](../advanced/raw-api.md) for that level of authoring.

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

Next, return to the [Concept map](README.md); Instance is the next concept in
the learning order and explains the live boundary.
