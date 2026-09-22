# Component

A **Component** is Fabric's semantic behavioral participant. It owns stable
behavior identity and may declare configuration, typed capability relations,
and a callable API without knowing which concrete realizations satisfy those
relations or make the Component live.

```text
Component = semantic behavior participant
```

A Component is not a Resource, System, Adapter, Instance, process, or service
locator. A Component may validly have zero Operations.

## Why Components exist

Components describe behavior, typed configuration, required capabilities, and
optional callable behavior without directly depending on an Adapter
implementation, provider module, runtime registry, Host machine, or placement.
This keeps declaration truth semantic while Composition and Core resolve
providers and a realization path creates participation.

## Definition, declaration, specification, participation

| Layer | Meaning |
| --- | --- |
| `ComponentDefinition` | Typed semantic definition: Config type, `ComponentId`, and declaration. |
| `ComponentDeclaration` | Runtime-free semantic truth: `ComponentId`, callable endpoint metadata, and named capability relations. |
| `ComponentSpec` | One configured declarative use, such as `Greeter::define(GreeterConfig { ... })`, including composition-facing requirement/provider-selection lowering. |
| `ComponentParticipation` | One active runtime incarnation, scoped to an Instance generation and participation identity. |

`component!` normally implements only the semantic definition. A declaration
is authoring convenience rather than ontology; handwritten definitions use the
same public machinery. The older `operations { ... handler ... }` frontend
temporarily remains as an explicit legacy self-realizing path while canonical
Component runtime authoring is converged separately.

A Component may be declaration-only or explicitly own native runtime
participation. Component realization internals are distinct from the canonical
Resource/System Adapter path: a Component's identity, Operations, and semantic
Resource/System requirements never become Adapter choice.

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

## Canonical declaration authoring

```rust
fabric::component! {
    NotesIndexer {
        id: "example.notes-indexer";

        config { prefix: String; }

        relations {
            requires {
                storage: NoteStore;
                clock: Clock;
            }
        }

        api {
            fn index(&self, note: Note) -> IndexResult;
        }
    }
}
```

Each section after `id` is optional. This macro declares semantic truth only:
it does not attach handlers, state, health, cleanup, or a runtime
participation. `api` describes callable behavior; it is not an implementation.
A local materialization therefore requires a separate realization path. Until
canonical Component runtime authoring arrives, the existing `operations`
frontend is the transitional self-realizing path and is not the recommended
declaration syntax.

A Component requires semantic Resources and Systems, not Adapters:

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

Relation roles are Component-local semantic names. `primary_store` and
`cache_store` remain distinct requirements even when both target `NoteStore`.
The target itself determines whether it is a Resource or System; canonical
authoring does not make the author repeat that fact. The existing lower-level
Resource/System carrier and binding machinery remains available to the
transitional realization path. Canonical declaration relations deliberately do
not create a runtime scope or a local binding by themselves.

When a realization is present, the runtime flow is:

```text
Component declares requirements
-> Composition selects providers
-> Core resolves and binds them
-> Component runtime receives typed dependencies
```

There is no Component registry lookup, Adapter search, or arbitrary service
lookup. The legacy self-realizing frontend generates typed dependency fields:

```rust
handler |dependencies, input: Input| async move { /* typed contracts */ }
handler |context, dependencies, input: Input| async move { /* also provenance */ }
```

## API, operations, and invocation

Canonical `api` methods use the same signature grammar as Resource and System
API declarations. Fabric deterministically lowers each method into the current
Component invocation metadata (`OperationDefinition`, `OperationId`, input and
output slots, and `OperationKey`) so ordinary authors do not write duplicate
operation or type-identity strings. That metadata remains invocation lowering,
not a second public Component authoring language.

The current `operations { ... }` block is legacy runtime authoring. It joins a
declaration to handler closures and therefore creates a self realization. It
remains supported only until a canonical realization language replaces it.

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

Canonical `component!` never generates that attachment. The legacy
`operations` frontend does: it obtains Core-resolved dependencies, constructs
the typed dependencies value, registers operations, and reports initial health
when materialized. This lower-level preparation occurs when that legacy
self-realizing Component is materialized, not when its declaration is added to
Composition.

After an Instance is running, activate Component participation explicitly:

```rust
let components = instance.components().expect("Component host");
components.materialize::<Greeter>()?;
let output = components.invoke_external(&greeter::operations::greet(), input).await?;
components.dematerialize::<Greeter>()?;
```

`Instance Running != every declared Component active`. Dematerialization ends
that active participation; it does not remove the Component declaration from
Composition. Participation passes through `Preparing` to `Active` and then
back to absence; its status carries participation, state, and local health.
The Component runtime host has its own `Starting`, `Ready`, `Stopping`, and
`Stopped` lifecycle. Host health remains independent, so `Ready + Healthy`,
`Ready + Degraded`, and `Ready + Unavailable` are all meaningful observations.
None is the lifecycle of one semantic Component declaration.

Normal host shutdown follows `Ready -> Stopping -> Stopped`. If Instance
startup is abandoned after host initialization, the same cleanup path is
`Starting -> Stopping -> Stopped`; `Stopping` remains the one deactivation
phase for both cases.

Preparation may establish runtime machinery owned by one participation. Normal
`component!` authoring may add an optional `teardown { ... }` section; direct
authors use `ComponentRuntimePreparation::with_teardown`. Fabric runs that
action exactly once after disabling participation-owned authority, on failed
preparation rollback, explicit dematerialization, or host stop. Base
preparation runs before augmentations; successful teardown runs in reverse
order, so later augmentation contributions clean up before the base.

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
