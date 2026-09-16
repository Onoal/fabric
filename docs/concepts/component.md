# Component

A **Component** is Fabric's semantic unit of behavior. It has stable behavior
identity, may declare typed Operations, and may require Resources and Systems
without knowing which concrete realizations satisfy those requirements.

```text
Component = semantic behavior participant
```

A Component is not a Resource, System, Adapter, Instance, process, or service
locator. A Component may validly have zero Operations.

## Why Components exist

Components describe behavior, typed configuration, required Resource
capabilities, required shared Systems, and typed Operations without directly
depending on an Adapter implementation, provider module, runtime registry,
Host machine, or placement. This keeps behavior semantic while Composition and
Core resolve the capability providers.

## Definition, declaration, specification, participation

| Layer | Meaning |
| --- | --- |
| `ComponentDefinition` | Typed semantic definition: Config type, `ComponentId`, and declaration. |
| `ComponentDeclaration` | Runtime-free semantic truth: `ComponentId`, Operations, Resource requirements, and System requirements. |
| `ComponentSpec` | One configured declarative use, such as `Greeter::define(GreeterConfig { ... })`, including requirements and provider-selection lowering. |
| `ComponentParticipation` | One active runtime incarnation, scoped to an Instance generation and participation identity. |

`component!` implements the normal semantic definition and explicit native
self-realization path, but it is authoring convenience rather than ontology.
Handwritten definitions use the same public machinery.

A Component may be declaration-only, explicitly self-realizing, or
Adapter-realized. These are realization choices for one declared behavior;
they do not change the Component's identity, Operations, or semantic
Resource/System requirements.

Component Config remains configuration of the semantic Component occurrence.
When an Adapter realizes that occurrence, Fabric supplies the configured
Component Config to the typed realization boundary; Adapter Config remains
configuration of the concrete implementation.

```text
ComponentDefinition
      | define(config)
      v
 ComponentSpec --added to--> Composition
      | Instance materializes and starts
      v
ComponentParticipation --registers--> typed invocation
```

`ComponentDeclaration` does not contain a live Instance, health, runtime
scope, active handler, or participation. A declaration with Operations says
endpoints exist; it does not say handlers are running.

## Identity and configuration

`ComponentId` is stable semantic behavior identity, for example
`notes.editor`. A low-level `Component` binds that identity to an `InstanceId`.
`ComponentParticipation` additionally carries `InstanceGeneration` and a
runtime-local `ComponentParticipationId`.

```text
ComponentId: notes.editor
        |
        +-- Instance local-a -- Participation / Generation 41
        `-- Instance local-b -- Participation / Generation 42
```

Participation identifiers are not global identities. They ensure stale runtime
authority cannot silently operate in a fresh generation.

Macro config is typed Component authoring/runtime-preparation input. It is not
Resource, Adapter, or System configuration.

## Normal authoring and dependencies

```rust
fabric::component! {
    Notes {
        id: "example.notes";
        config { prefix: String; }
        requires { storage: NoteStore(provisional); }
        system { operations: OperationsSystem(version = "^1"); }
        operations {
            // typed endpoints
        }
    }
}
```

Each section is optional. A Component requires semantic Resources and Systems,
not Adapters:

```text
Component -> Resource/System
not Component -> [Adapter](adapter.md)
```

Requirement names are Component-local roles. In `primary_store` and
`cache_store`, the names explain what each dependency means to the Component;
they do not rename selected occurrences such as `NoteStore("primary")` and
`NoteStore("cache")`. [Composition](composition.md) selects and binds those
occurrences. [Systems](system.md) remain a distinct semantic world with one
coherent normal typed occurrence per `SystemId`.

At runtime the flow is:

```text
Component declares requirements
-> Composition selects providers
-> Core resolves and binds them
-> Component runtime receives typed dependencies
```

There is no Component registry lookup, Adapter search, or arbitrary service
lookup. For requirements, `component!` generates typed dependency fields:

```rust
handler |dependencies, input: Input| async move { /* typed contracts */ }
handler |context, dependencies, input: Input| async move { /* also provenance */ }
```

## Operations and invocation

An Operation has an `OperationId`, semantic input/output type identities, typed
Rust input/output, and a runtime handler. `OperationDefinition` is not a
running handler. `OperationKey<Input, Output>` carries both Rust typing and the
semantic operation/input/output identities agreed by caller and runtime.

The supported normal handler shapes are:

```text
|input|
|dependencies, input|
|context, input|
|context, dependencies, input|
```

`context: invocation;` opts into `InvocationContext`, which carries
`InstanceId`, `InstanceGeneration`, `InvocationId`, and root
`InvocationOrigin`. It is provenance—not authentication, authorization,
identity, tracing, network metadata, or request headers. Current origins are
`External` and `Component(Component)`; the normal high-level path is external
invocation, and Fabric 0.1 does not define declarative Component-to-Component
dependencies.

### Domain outcome versus runtime error

Domain failure belongs in the typed Operation output. Fabric runtime/control
failure belongs in `ComponentError`:

```rust
output: Result<Document, DocumentError> = "example.documents.open.outcome";
```

External invocation therefore intentionally yields:

```text
Result<Result<Document, DocumentError>, ComponentError>
```

The inner result may be `DocumentNotFound`, validation failure, or another
package-owned outcome. The outer error covers Fabric concerns such as an
unavailable Component, operation type mismatch, missing local realization, or
stale generation/participation. Fabric does not merge these into a universal
application error.

## Native realization and participation

`ComponentDeclaration != ComponentRuntimeDefinition`. A `ComponentSpec` may
carry no local self-realization, so declaration-only Components—including
zero-operation Components—are valid semantic truth. They are known to the
Component environment, but native local materialization without a realization
fails with the existing `MissingComponentRuntimeAttachment` error.

Normally `component!` generates an attachment that obtains Core-resolved
dependencies, constructs the typed dependencies value, registers operations,
and reports initial health. This lower-level preparation occurs when the
Component is materialized, not when its declaration is added to Composition.

After an Instance is running, activate Component participation explicitly:

```rust
let components = instance.components().expect("Component host");
components.materialize::<Greeter>()?;
let output = components.invoke_external(&greeter::operations::greet(), input).await?;
components.dematerialize::<Greeter>()?;
```

`Instance Running != every declared Component active`. Dematerialization ends
that active participation; it does not remove the Component declaration from
Composition. Participation passes through `Preparing` to `Active`; its status
carries participation, state, and local health. The Component runtime host has
its own `Starting`, `Ready`, `Degraded`, `Stopping`, and `Stopped` lifecycle;
that is not the lifecycle of one semantic Component declaration.

## Build time and runtime

| Concept | Declaration time | Runtime |
| --- | --- | --- |
| `ComponentId` | declared | preserved |
| Config | authoring input | captured by attachment |
| Resource/System requirements | declared | typed resolved values |
| `OperationDefinition` | declared | handler registered |
| `ComponentParticipation` | absent | generation-scoped |
| `InvocationContext` | absent | per invocation |

## External augmentation

An external semantic may attach to a configured `ComponentSpec` without
changing the `ComponentDefinition`. The Component continues to own its base
Config, identity, and `ComponentDeclaration.operations`: an augmentation may
not append an undeclared base operation.

```rust
let audit = Greeter::define(GreeterConfig {})
    .augment::<Audit>(())?
    .using(AuditSupport);
let audit_requirement = audit.requirement();

let traced = audit.into_set()
    .augment::<Trace>(())?
    .using(TraceSupport);
let trace_requirement = traced.requirement();

let built = Fabric::new("example.component-augmentation")?
    .component(traced)
    .build()?;
# let _ = (audit_requirement, trace_requirement, built);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`ComponentAugmentationDefinition` owns only the added semantic `X` and its
typed contract. `ComponentAugmentationSupportDefinition` independently
provides that contract and may prepare the same Component participation. A
Component has one base runtime attachment and zero or more augmentation
preparation contributions; neither a second Component participation nor a
second base realization is created. This applies to zero-operation Components
and to ordinary self-realizing or Adapter-realized Components.

`ComponentAugmentationSet` keeps one configured Component while multiple
distinct semantics are attached. Each supported attachment has a typed,
target-bound `ComponentAugmentationRequirement`; `without_support()` preserves
a bare attachment but deliberately offers no provider-bound requirement. This
does not create a primary Component semantic contract or a
Component-to-Component dependency. See [Augmentation](augmentation.md).

## Boundaries

```text
Component = behavior
Resource  = technical capability occurrence
System    = shared instance-wide capability
Adapter   = realization of semantics
Instance  = whole live Composition materialization
```

A Component is not an OS process, microservice, container, thread, VM, or
network service. It does not choose provider implementations, create Resource
occurrences, or own Instance start/stop. Lower-level Component control and
registry rails exist for advanced integrations, but ordinary authoring uses
definition, declaration, requirements, operations, materialization, and the
bounded Instance Component surface.

Next: [Resource](resource.md), the semantic technical capability boundary.
Return to the [Concept map](README.md) or the precise
[Architecture](../architecture.md).
