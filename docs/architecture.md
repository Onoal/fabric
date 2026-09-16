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
Resources, Systems, Components, and Component Resource/System bindings. A
Resource binding identifies the selected `ResourceId + ResourceName`.

`manifest.diagnostics()` is the explicit raw/Core view for backing Module
declarations, provider selections, Blocks, and Composition exports. Manifest
is not a deployment, package, serialization, reconstruction, or runtime-state
format.

`Module` is Core structural/runtime machinery, not a normal high-level semantic
world. `Block` is advanced grouping and reporting machinery, not dependency
topology. Core graph truth—not Block order—determines dependency and lifecycle
ordering.

The normal authoring path is `fabric` and `fabric::prelude`. Advanced
capability remains public through named modules including
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

The repository contains experimental implementation research under
`experimental/`. These modules are not part of the normal Fabric SDK or the
Fabric 0.1 public model.

## Scope

Fabric 0.1 does not define a scheduler, placement/capacity engine, global
Resource/Adapter/requirement registry, package manager, portable deployment
language, Manifest reconstruction, dynamic plugin ABI, IDL generation,
container orchestration, or identity/authority model.
