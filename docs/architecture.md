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
- **Adapter** realizes an adaptable Component, Resource, or System. It names both the
  semantic target and that target's public typed realization interface.
- **Component** owns semantic behavior expressed as typed operations. It can
  require Resources and Systems.
- **Host** represents environmental compatibility for live Adapter
  materialization. It is not a scheduler, placement engine, capacity model, or
  device/cloud ontology.
- **Composition** is a reusable declaration. An **Instance** is one live,
  generation-scoped materialization. `Composition != Instance`.

Fabric generalizes machinery rather than package or product vocabulary.

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
its own special runtime rule: one base `ComponentRuntimeDefinition` plus zero
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
fresh generation in `Ready`; `start()` reaches `Running`; `stop()` reaches
`Stopped`, which is terminal for that generation. Lifecycle is separate from
Health, and both are separate from Component runtime lifecycle, readiness, and
desired Component control.

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

A Component, Resource, or adaptable System may expose a public realization trait. An
Adapter names both sides explicitly:

```rust
fabric::adapter! {
    ExampleAdapter
        for resource ExampleResource
        implements ExampleResourceRealization
    {
        // schema, realization version, typed config, and runtime methods
    }
}
```

Compatibility states what can satisfy a requirement. Provider selection chooses
one compatible provider for a Composition. A Component declares semantic needs;
it does not choose an Adapter.

## Components and operations

Components declare Resource dependencies in `requires {}` and System
dependencies in `system {}`. A Component Resource requirement is owned by the
Component and identified by `ComponentId + local requirement name`. Its local
role is distinct from the selected Resource's `ResourceName`, so roles with the
same target type can bind independently to different Resource occurrences.

Handlers receive typed resolved contracts through a dependencies value. Config
is Component-owned lexical configuration; it is distinct from dependencies,
invocation context, and operation input. Declarative Component-to-Component
dependencies are outside Fabric 0.1.

An operation is a typed `Input -> Output` endpoint. Its output may be a
package-owned `Result<Success, DomainError>`. `ComponentError` is the outer
runtime/control plane, so external invocation can yield
`Result<Result<Success, DomainError>, ComponentError>` without erasing domain
typing.

`context: invocation;` opts an operation into a runtime-supplied
`InvocationContext`. The context contains invocation provenance: InstanceId,
InstanceGeneration, InvocationId, and root InvocationOrigin. It is not an
identity, authority, authentication, tracing, or network-metadata model.

## Runtime boundary

Composition can explicitly export a bounded typed runtime capability. Core
retains only declared exports for one materialization; Instance does not expose
arbitrary contract lookup. A Component-bearing high-level Fabric build exports
one Component operational capability.

`BuiltFabric` materializes to `FabricInstance`, which delegates identity,
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
topology. Core graph truth—not Block order—determines dependency and lifecycle
ordering.

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
future minor release while remaining compatibility-sensitive in a `0.2.x`
patch line. Binding and Resource Registry research remain repository-only.

## Scope

Fabric's completed 0.3 Time / Lifecycle / Change milestone does not define a scheduler, placement/capacity engine,
global Resource/Adapter/requirement registry, package manager, portable
deployment language, Manifest reconstruction, dynamic plugin ABI, IDL
generation, container orchestration, identity/authority model, lifecycle
manager, universal lifecycle state machine, replacement API, migration engine,
recovery controller, rollback engine, event history, or durable runtime state.
