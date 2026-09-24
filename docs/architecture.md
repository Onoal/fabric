# Fabric architecture

This document describes Fabric's public model. Start with
[Getting Started](getting-started.md) and the [Concepts overview](concepts/README.md)
for the learning path. The [Advanced Raw API](advanced/raw-api.md) describes
direct Core authoring.

## Model and ownership

- **Core** validates the graph, evaluates compatibility, selects providers, and
  materializes Instances. It is the structural resolver.
- **Resource** is an occurrence-based technical capability. An occurrence is
  identified by `ResourceId + ResourceName`.
- **System** is an instance-wide shared capability. Normal typed authoring has
  one coherent occurrence per `SystemId` in a Composition.
- **Adapter** is concrete realization machinery for a Resource or System. In
  normal authoring it names its target once; that target supplies the typed
  realization requirement. Component participation remains a distinct model.
- **Component** is a semantic behavioral participant. It owns identity and
  may declare Config, Relations, and a typed API; a self runtime or Adapter
  realization prepares its live participation.
- **Host** represents environmental compatibility for live Adapter
  materialization. It is not a scheduler, placement engine, capacity model, or
  device/cloud ontology.
- **Composition** is a reusable declaration. An **Instance** is one live,
  generation-scoped materialization. `Composition != Instance`.

Fabric generalizes machinery rather than package or product vocabulary.

## Semantic subjects and live realization

```text
Self realization
Resource/System semantic API <- self-owned live runtime

Direct Adapter realization
Resource/System semantic API <- Adapter-owned live provider

Differential realization
Resource/System semantic API <- semantic assembly
                                      ^
                                      | typed effective realization requirement
                                Adapter-owned live provider
```

The first is appropriate only when the semantic subject truly owns live
behavior. The second is the normal adapted form. The third is explicit semantic
mediation: Core still selects one provider for the semantic API and one provider
for the effective realization Contract, never a provider per method.

```text
typed requirement -> selected provider -> bind typed Contract -> consumer
Composition        -> materialize       -> Instance generation
```

The same resolved graph supplies dependency ordering, startup, and reverse
cleanup. It is not a service locator or a second resolver.

## Open semantic augmentation

An external semantic augmentation is a typed addition to a selected Resource,
System, or Component. The base subject, augmentation semantic, attachment,
support implementation, and realization are distinct truths. The base does not
need to know an augmentation; an augmentation may know its base; independently
authored support may know both.

```text
base subject != X != attachment != support implementation != realization
```

Resource attachment targets `ResourceId + ResourceName`. System attachment
targets the selected `SystemId` occurrence; normal typed authoring has one
coherent occurrence per identity and no `SystemName`. Component attachment
targets a configured `ComponentSpec` under its `ComponentId`. Component keeps
its own special runtime rule: one base `ComponentParticipationRealization` plus zero
or more augmentation preparation contributions operating on the same
generation-scoped participation. Augmentation never alters the base Component
operation declaration.

Support implementation truth—its dependencies, extra provided contracts, and
Host compatibility—remains implementation truth. Fabric combines it with the
target relationship and augmentation provision in one ordinary Core provider
occurrence. Consumer requirements name the typed semantic contract owned by
`X`, not the support implementation. A target plus augmentation Contract
identity is one attachment identity: `X + Y` is distinct, while repeating `X`
on the same target is rejected rather than creating a second semantic subject.

The Manifest records those semantic attachments with explicit Resource,
System, and Component entry types. It does not treat provider module IDs,
support dependencies, Host requirements, or runtime history as semantic
attachment fields. `manifest.diagnostics()` exposes the separate raw/Core
graph when that implementation detail is needed.

This is open-world authoring, not inheritance, metadata, a global semantic
registry, or a second resolver. All paths lower through existing Core Modules,
Contracts, requirements, provider selections, dependency ordering, and
materialization. Adapter remains realization machinery rather than an
augmentation target; Host remains environmental compatibility truth. The
independent `verification/third-party-augmentation` workspace proves the
public ownership direction across base, semantic, support, consumer, and app
crates.

## Time, lifecycle, and declarative change

`InstanceId` is a caller-selected logical runtime name. `InstanceGeneration`
identifies one Fabric-minted, process-local runtime incarnation under that
name. It is not a software version, Composition revision, durable epoch, or
globally unique sequence. A Composition and an Instance are distinct: runtime
failure, start, stop, and health observation do not mutate declarative
Composition truth.

Core's Instance lifecycle is deliberately narrow: materialization creates a
fresh generation in `Ready`; `start()` reaches `Running`; fallible `stop()`
deterministically cleans every materialized runtime participant and reaches
`Stopped`, which is terminal for that generation. Startup abandonment also uses
that cleanup path. Lifecycle is separate from Health, and both are separate
from Component runtime lifecycle, readiness, and desired Component control.

Runtime participation is a cross-cutting execution spine, not another semantic
subject. Configuration is declarative materialization input; `RuntimeState` is
fresh ephemeral state for one materialized occurrence; lifecycle is that
occurrence's initialize/start/stop participation; Health reports its current
ability to fulfill responsibility. A Resource/System owns state and lifecycle
only when it explicitly self-realizes or mediates semantic behavior. A normal
adapted subject owns neither a fake forwarding runtime nor Adapter machinery:
the Adapter owns concrete state, lifecycle hooks, and health. Contract handles
for one occurrence share its live state, while a fresh materialization receives
fresh state.

Structural declaration or realization change is represented by authoring new
declarative truth and materializing a fresh Instance generation. Fabric does
not infer supersession, migration, state transfer, cutover, rollback, or a
replacement authority. Compatible contracts, schemas, and Hosts establish that
a selection can bind or materialize; they do not establish replacement safety.
`InstanceReport` is bounded current observation, not event history or a durable
runtime record.

## Relations and realization

Typed Contracts make consumption explicit. Resource, System, Adapter, and
Component authoring lowers through Core requirements, providers, and provider
selections. There is no second resolver or general runtime service locator.

A Component may expose a public realization trait for advanced authoring. For a
normal Resource or System, the target API is the direct realization contract and
the Adapter names the target once:

```rust
fabric::adapter! {
    ExampleAdapter for ExampleResource {
        id: "docs.example-adapter";
        // optional typed Config and implementation of ExampleResource's API
    }
}
```

Compatibility states what can satisfy a requirement. Provider selection chooses
one compatible provider for a Composition. A Component declares semantic needs;
it does not choose an Adapter.

## Provider composition and differential realization

Core resolves one selected provider for each typed Contract. A provider may
derive one typed capability from typed required capabilities; this is contract
composition, not method-level selection. Differential realization uses that
general rule only when a semantic owner deliberately mediates part of its API:

```text
semantic API <- semantic assembly <- effective realization <- Adapter
```

The semantic assembly exports the complete semantic API, implementing mediated
methods and typed-delegating direct methods. The Adapter exports the effective
realization contract. A direct Adapter realization does not add this layer.
Neither form recreates a mandatory Resource/System forwarding runtime.

## Components, API, and invocation lowering

A Component is a semantic behavioral participant. Its canonical declaration is
`id` plus optional `config`, `relations`, and `api` sections. A relation is a
Component-local role targeting either a Resource or System; the target's type
supplies the capability requirement. The role is distinct from a selected
Resource occurrence, so two roles can bind independently to the same target.

`api` declares typed callable behavior only. Current Component invocation
machinery lowers API methods to deterministic operations and typed keys; it is
not a second declaration language and does not make the Component live. A
declaration-only Component is valid in Composition, but participation requires
a realization.

Canonical `runtime` supplies a default self realization for one participation;
canonical Component-target `adapter!` supplies the same realization boundary.
Runtime implementations use typed `self.config()` and `self.relations()`
accessors. `InvocationContext` remains advanced invocation provenance—not
semantic API input, identity, authority, authentication, tracing, or network
metadata.

## Runtime boundary

The normal build boundary is explicit:

```text
Fabric
  ↓ build
SDK Composition
  ├── FabricManifest (semantic/provenance inspection)
  └── fabric::core::Composition (resolved generic structure)
        ↓ materialize
      FabricInstance
```

`Composition::core()` is the deliberate advanced escape hatch; normal code
continues through the SDK Composition.

Composition can explicitly export a bounded typed runtime capability. Core
freezes each declared export's provider ownership at build; each Instance then
retains that selected provider's live value for its own materialization.
Instance does not expose arbitrary contract lookup. A Component-bearing
high-level Fabric build exports one Component operational capability.

`Composition` materializes to `FabricInstance`, which delegates identity,
generation, lifecycle, and reporting to the Core Instance. Its optional
`FabricComponents` façade materializes and dematerializes Components and
invokes typed `OperationKey<I, O>` values. Handles are local to one Instance
and generation. A Resource/System-only Composition has no Component host.

## Inspection and advanced APIs

`FabricManifest` is immutable semantic Composition inspection. It exposes
Resources, Systems, Components, Component Resource/System bindings, and
explicit semantic augmentation attachments. A Resource binding identifies the
selected `ResourceId + ResourceName`.

`manifest.diagnostics()` is the explicit raw/Core view for backing Module
declarations, provider selections, Blocks, and Composition exports. Manifest
is not a deployment, package, serialization, reconstruction, or runtime-state
format.

`Module` is Core structural/runtime machinery, not a normal high-level semantic
world. `Block` is advanced grouping and reporting machinery, not dependency
topology. Core graph truth determines provider-before-consumer dependency and
lifecycle ordering; flattened Block/module insertion order is only the current
deterministic tie-break for otherwise independent modules.

The normal authoring path is `fabric::*`; `fabric::prelude::*` remains an
optional compatibility convenience. Advanced capability remains public through named modules including
`fabric::authoring`, `fabric::core`, and `fabric::component`.
Handwritten third-party definitions and typed realization interfaces use the
same public machinery as macro-generated definitions.

An ordinary Rust function is the supported Composition reuse mechanism:

```rust
pub fn local_stack() -> fabric::Fabric {
    fabric::Fabric::new("example").expect("valid composition id")
}
```

Callers can extend the returned `Fabric` before `build()`.

## Experimental modules

`fabric::experimental::projection` is explicitly shipped for exploration and
use, but is not canonical Core ontology, a Composition participant, or a
Manifest semantic category. Experimental APIs may change or disappear at a
future minor release while remaining compatibility-sensitive within a patch
line. Binding and Resource Registry research remain repository-only.

## Scope

Fabric 0.4 does not define a scheduler, placement/capacity engine,
global Resource/Adapter/requirement registry, package manager, portable
deployment language, Manifest reconstruction, dynamic plugin ABI, IDL
generation, container orchestration, identity/authority model, lifecycle
manager, universal lifecycle state machine, replacement API, migration engine,
recovery controller, rollback engine, event history, or durable runtime state.
