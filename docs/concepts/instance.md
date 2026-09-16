# Instance

An **Instance** is one live materialization of a validated Composition. It has
its own runtime identity, generation, lifecycle, materialized runtime modules,
explicit exports, and operational state.

```text
Composition describes.
Instance lives.
```

The two must not be confused:

```text
Composition != Instance
```

Composition owns declarative truth. Instance owns one runtime incarnation of
that truth.

## The temporal contract

An Instance is one generation-scoped runtime incarnation of a Composition
under an `InstanceId`:

```text
materialize -> fresh InstanceGeneration -> Ready
start       -> Running
stop        -> Stopped (terminal for that generation)
```

`InstanceId` is the logical name selected by the caller. `InstanceGeneration`
identifies one runtime incarnation under that name. Reusing the same
`CompositionId` and `InstanceId` in a later materialization always creates a
fresh generation; it never revives the stopped Instance object. Generations
are Fabric-minted, process-local runtime identities. They are not software
versions, Composition revisions, durable epochs, or globally unique values.

`LifecycleState` intentionally remains only `Ready`, `Running`, and
`Stopped`. A failed initialize or start operation returns its error and leaves
that generation `Stopped`; failure is not a second durable lifecycle state.
Cleanup stops already initialized runtime modules, but is not a promise of
generic rollback for arbitrary external side effects.

## Why Instance exists

A Composition can be validated, inspected, reused, and materialized more than
once without itself becoming runtime state. Each materialization gets its own
`InstanceId`, `InstanceGeneration`, lifecycle, runtime modules, runtime
exports, and health/reporting state. This makes declaration reuse possible
without accidentally sharing live state.

```text
Composition
     |
     +-- materialize("local-a") --> Instance A
     |
     `-- materialize("local-b") --> Instance B
```

Both Instances originate from the same declaration. A change in one live
Instance is not a change to the Composition.

## Materialize a Composition

The normal high-level path begins with `Fabric`, builds a `BuiltFabric`, and
then materializes a `FabricInstance`:

```rust
use fabric::prelude::*;

let built = Fabric::new("example.instance")
    .expect("valid CompositionId")
    // declarations go here
    .build()
    .expect("valid Composition");

let mut instance = built
    .materialize_named("example.instance.local")
    .expect("materialize");
```

`materialize_named` is appropriate when the Composition has no declared Host
materialization requirement. When a realization declares Host compatibility
requirements, supply the concrete `HostDescriptor` at materialization:

```rust
let instance = built.materialize_named_on("example.instance.local", &host)?;
```

The Composition declares compatibility requirements; the `HostDescriptor` is
provided for validation while materializing. The Host does not become Instance
identity or a service discovered from the Instance.

### Materialize is not start

Materialization creates a live Instance in `LifecycleState::Ready`. It does
not start it.

```text
Composition -> materialize -> Ready Instance -> start -> Running Instance
                                               -> stop  -> Stopped Instance
```

### Example A: materialize, start, and stop

Continuing the materialization above:

```rust
assert_eq!(instance.lifecycle(), LifecycleState::Ready);

instance.start().expect("start");
assert_eq!(instance.lifecycle(), LifecycleState::Running);

instance.stop();
assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
```

`stop()` is also valid from `Ready`, which transitions directly to `Stopped`.
Calling it again after `Stopped` leaves the Instance stopped. `start()` is
valid only from `Ready`; a stopped Instance is not restarted. To run again,
create a fresh materialization.

## Core Instance and FabricInstance

`core::Instance` is the lower-level live Core realization. `FabricInstance` is
the normal high-level Fabric façade around it.

```text
FabricInstance = normal lifecycle, report, and bounded operation surface
core::Instance = lower-level runtime machinery
```

The high-level façade provides:

```text
instance_id()  generation()  lifecycle()  report()
start()        stop()        components()
core()         core_mut()
```

`core()` and `core_mut()` remain available for deliberate low-level work, but
normal users should prefer the `FabricInstance` lifecycle/reporting methods
and its bounded Component surface. See the [Advanced Raw API](../advanced/raw-api.md)
for direct Core authoring.

## Three identities

These identifiers refer to different things:

| Identifier | Meaning |
| --- | --- |
| `CompositionId` | Which declaration this Instance came from. |
| `InstanceId` | The named live Instance. |
| `InstanceGeneration` | This specific materialized runtime incarnation. |

```text
CompositionId != InstanceId != InstanceGeneration
```

Core mints a fresh `InstanceGeneration` for every materialization. The
generation distinguishes separate runtime incarnations; it is not a
Composition, package, schema, deployment, or user-selected version.

### Example B: two Instances from one Composition

```rust
let first = built.materialize_named("example.instance.first")?;
let second = built.materialize_named("example.instance.second")?;

assert_ne!(first.instance_id(), second.instance_id());
assert_ne!(first.generation(), second.generation());
```

One Composition can create independent runtime state for both Instances.

### Example C: same InstanceId, fresh generation

The same logical `InstanceId` does not mean the same runtime incarnation:

```rust
let first = built.materialize_named("example.instance.local")?;
let second = built.materialize_named("example.instance.local")?;

assert_eq!(first.instance_id(), second.instance_id());
assert_ne!(first.generation(), second.generation());
```

Generation prevents provenance from confusing an earlier local Instance with a
later materialization using the same name. It is also available to
invocation-provenance machinery, without making this page an InvocationContext
reference.

`InstanceId` itself is not a machine identity, network endpoint, placement,
Host identity, or scheduler allocation. It identifies a Fabric Instance.

## Lifecycle and runtime ordering

The current lifecycle is intentionally small:

```text
Ready --start()--> Running --stop()--> Stopped
Ready --stop()-----------------------> Stopped
Stopped --stop()---------------------> Stopped
```

Starting follows the validated dependency order from the Composition's
declaration graph: providers are initialized and started before dependent
runtime modules. Stopping uses the reverse dependency order. Neither order is
defined by Block order, builder insertion order, or source-file order; see the
[Composition Block clarification](composition.md#advanced-structural-machinery).

If initialization or startup fails, Fabric stops already initialized runtime
modules, transitions the Instance to `Stopped`, and returns `InstanceError`.
It does not leave a failed start pretending to be `Running`, and it does not
automatically retry that Instance.

## Live state and runtime inspection

At the Core level, an Instance carries Composition provenance, `InstanceId`,
`InstanceGeneration`, `LifecycleState`, materialized runtime blocks/modules,
resolved runtime exports, and runtime health/reporting state. These are live
materialization facts, not mutable copies of the original semantic authoring
objects.

`instance.report()` returns an `InstanceReport` with:

```text
composition_id
instance_id
generation
lifecycle
health
runtime block reports
```

`InstanceReport` is runtime inspection. `FabricManifest` is semantic
Composition inspection:

```text
FabricManifest                 InstanceReport
--------------                 --------------
What was declared?             What materialization is this?
What semantic bindings exist?  What generation is it?
                               What lifecycle and health state is live?
```

```text
FabricManifest != InstanceReport
```

An `InstanceReport` is a current bounded observation, not an event history,
transition journal, durable lifecycle record, or timestamped monitoring feed.
Its lifecycle and health fields remain separate: lifecycle says where the
runtime incarnation is in its bounded execution path, while health aggregates
the runtime modules' current condition.

Health is local runtime inspection. An Instance report aggregates block health:
`Unavailable` takes precedence, then `Degraded`, otherwise the report is
`Healthy`. It does not define distributed monitoring, readiness probes, SLA
state, automatic recovery, or remote health management.

### Example E: inspect declaration and runtime separately

```rust
let manifest = built.manifest();
assert_eq!(manifest.components().len(), 1);

let report = instance.report();
assert_eq!(report.instance_id, instance.instance_id().clone());
assert_eq!(report.generation, instance.generation());
assert_eq!(report.lifecycle, instance.lifecycle());
```

## Explicit exports, not a service locator

Core retains runtime capability only through an explicitly declared
`CompositionExport<T>`. It does not retain every module contract for arbitrary
lookup.

```text
Instance != generic service locator
```

There is intentionally no normal `instance.get::<Anything>()` or arbitrary
type-based runtime-service lookup. Runtime capability access is bounded and
deliberate.

Each materialization resolves and retains its own runtime export values. Two
Instances from the same Composition should therefore be understood as separate
runtime realizations, not one shared live runtime.

## The bounded Component surface

`FabricInstance::components()` returns `Option<&FabricComponents>`. Not every
Composition contains the native Component runtime host, so a Resource/System-
only Composition returns `None` rather than a fake Component control surface.

```text
Component is not mandatory for Instance existence.
```

The Component host is not the Instance itself. A `FabricInstance` may expose
that bounded control surface when its Composition contains the host; the
Instance remains the broader live materialization.

### Example D: Component participation is separate from Instance start

```rust
instance.start().expect("start Instance runtime");

let components = instance.components().expect("Component host");
components.materialize::<Greeter>().expect("materialize Greeter");
// The Component can now be invoked through components.invoke_external(...).

components.dematerialize::<Greeter>().expect("dematerialize Greeter");
instance.stop();
```

Starting an Instance does not materialize every declared Component. Component
participation is explicit and has its own boundary. The dedicated Component
page will cover operations and Component lifecycle in depth.

## What an Instance is not

An Instance materializes declared truth; it does not define Resource, System,
Component, Adapter, or provider-selection semantics. It does not own global
scheduling, placement, distributed orchestration, identity/authority, network
reachability, deployment/package serialization, or a global registry.

```text
Instance != deployment specification
Instance != Host
Instance != Component
Instance != OS process, container, VM, thread, or machine
```

A Host may be evaluated during materialization, but an Instance is what was
materialized. An implementation may use runtime machinery internally, but
Fabric's Instance remains the semantic live-materialization boundary.

Next: [Component](component.md), Fabric's semantic behavior participant. You
can also return to the [Concept map](README.md) or continue to the precise
[Architecture](../architecture.md).
