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

## Identity, generation, and lifecycle

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
Cleanup calls fallible `stop` on every successfully materialized runtime
participant, including a participant whose context binding, initialization, or
start did not complete. Failures preserve the primary startup error and remain
observable as cleanup evidence; this is not a promise of generic rollback for
arbitrary external side effects.

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

## Materialization profile

Materialization also has occurrence-specific intent:

```text
Composition != MaterializationProfile != Host != MaterializationPlan != Instance
```

`MaterializationProfile` is frozen provenance for one materialized occurrence.
It names intent such as `default`, `diagnostic`, or `production` without
changing the Composition or describing Host facts. In v1 it is intentionally
bounded to identity/provenance; it does not select Adapters, schedule
deployment, or run a realization planner.

The simple materialization path uses the canonical default profile:

```rust
let instance = composition.materialize("example.instance.local")?;
assert_eq!(instance.materialization_profile().name().as_str(), "default");
```

An explicit profile can be supplied when occurrence intent should be visible on
the resulting Instance:

```rust
let profile = MaterializationProfile::new("diagnostic")?;
let instance = composition.materialize_with_profile(
    "example.instance.diagnostic",
    &profile,
)?;
assert_eq!(instance.observe().materialization_profile(), &profile);
```

Profile is not Host. Host describes environmental facts and compatibility
inputs. Profile describes occurrence intent. The same Composition may be
materialized with different Profiles and different Hosts without mutating
Composition inspection.

## Materialization plan

Before a live Instance exists, Fabric can prepare the effective non-live
materialization truth:

```text
Composition
    + MaterializationProfile
    + Host
        -> MaterializationPlan
        -> Instance
```

`MaterializationPlan` is immutable and inspectable. It records which
Composition is being materialized, which Profile applies, which Host context
was validated, the current v1 realization truth inherited from Composition
resolution, and the initial Component participation intent that will seed a
fresh Instance.

It is not live truth:

```text
MaterializationPlan != InstanceGeneration
MaterializationPlan != LifecycleState
MaterializationPlan != Health
MaterializationPlan != Adapter runtime State
MaterializationPlan != ComponentParticipation
```

Creating a Plan does not materialize runtime modules. Generation and runtime
state appear only when the Plan is materialized:

```rust
let profile = MaterializationProfile::new("diagnostic")?;
let plan = composition.plan_with_profile_on(&profile, &host)?;

assert_eq!(plan.materialization_profile(), &profile);
assert_eq!(plan.host(), Some(&host));

let instance = plan.materialize("example.instance.diagnostic")?;
assert_eq!(instance.materialization_plan(), plan.provenance());
```

Existing `composition.materialize...` methods are sugar over this boundary.
Normal users can keep the simple path; advanced users can create and inspect a
Plan before creating an Instance.

One Plan is reusable because it is non-live. Each materialization of that Plan
receives a fresh `InstanceGeneration` and fresh runtime state. To change
Profile or Host intent, create another Plan.

## Instance facilities

An Instance is also the owner of optional live-only operational machinery:

```text
Instance
├── semantic live world
│   ├── Resources
│   ├── Systems
│   ├── ComponentParticipations
│   └── Adapter runtimes
│
└── InstanceFacilities
```

`InstanceFacility` exists only around one live Instance generation. It can be
attached after materialization to observe bounded Instance lifecycle events
such as attach, start, stop, and detach. It is appropriate for operational
tooling like local diagnostics, profiling, tracing bridges, or runtime
inspection.

It is not semantic system truth:

```text
InstanceFacility != Resource
InstanceFacility != System
InstanceFacility != Component
InstanceFacility != Adapter
InstanceFacility != MaterializationPlan
```

Instance-wide scope alone does not make something a System. A semantic logging
capability required by application code may be a Resource or System; an
operator-attached diagnostic recorder for one live occurrence is an
InstanceFacility.

Facilities are generation-scoped and optional. Attaching a Facility does not
change the Composition, MaterializationPlan, MaterializationProfile,
`InstanceId`, or `InstanceGeneration`. Running attachment is supported and
observes the current Running context without fabricating historical start
events. A terminal Stopped generation rejects new Facility attachment.

Generic observation reports only Facility presence:

```rust
let observation = instance.observe();
let names = observation.facilities();
```

Facility-private state is not dumped into `InstanceObservation`. Facilities do
not redefine Instance health in v1 and do not intercept Component invocation,
Resource API calls, System API calls, or Adapter operations. V1 intentionally
does not introduce a generic event bus, middleware layer, Facility dependency
graph, or Profile-driven Facility policy.

## Materialize a Composition

The normal high-level path begins with `Fabric`, builds a `Composition`, and
then materializes an `Instance`:

```rust
use fabric::*;

let composition = Fabric::new("example.instance")
    .expect("valid CompositionId")
    // declarations go here
    .build()
    .expect("valid Composition");

let mut instance = composition
    .materialize("example.instance.local")
    .expect("materialize");
```

`materialize` is appropriate when the Composition has no declared Host
materialization requirement. When a realization declares Host compatibility
requirements, supply the concrete `HostDescriptor` at materialization:

```rust
let instance = composition.materialize_on("example.instance.local", &host)?;
```

The Composition declares compatibility requirements; the `HostDescriptor` is
provided for validation while materializing. The Host does not become Instance
identity or a service discovered from the Instance.

An explicit profile can also be combined with a Host:

```rust
let profile = MaterializationProfile::new("diagnostic")?;
let instance = composition.materialize_with_profile_on(
    "example.instance.diagnostic",
    &profile,
    &host,
)?;
```

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

instance.stop().expect("stop");
assert_eq!(instance.lifecycle(), LifecycleState::Stopped);
```

`stop()` is also valid from `Ready`; it performs the same participant cleanup
before reaching `Stopped`, and reports any cleanup failures. Calling it again
after `Stopped` leaves the Instance stopped. `start()` is valid only from
`Ready`; a stopped Instance is not restarted. To run again, create a fresh
materialization.

## Semantic observation

The normal live read model is `Instance::observe()`. It combines the immutable
semantic Composition context with the current live Core runtime report and the
current Component host observations.

```text
Composition = complete declared semantic truth
Instance    = complete current live semantic observation boundary
```

`observe()` reports semantic Resources, Systems, Components, realization
observations, Component desired participation, observed participation, observed
health, aggregate lifecycle, and aggregate health. It is a current projection,
not a history, journal, durable runtime record, or second source of truth.

Component desired and observed state may intentionally differ:

```text
Enabled  + Absent
Enabled  + Active
Disabled + Active
Disabled + Absent
```

`start()` starts the Instance lifecycle. It does not reconcile Component
participation. Use the high-level Component handle or `reconcile_components()`
to converge desired Component participation with observed participation.

## Instance-bound Component API

The normal Component path is bound to one Instance generation:

```rust
let app = instance.component::<App>()?;
app.reconcile()?;
let output = app.some_semantic_operation(input).await?;
```

For canonical `component!` declarations, Fabric generates typed methods on a
local extension trait for the instance-bound handle. Handwritten Components can
use the same handle with `call(&OperationKey, input)` when they deliberately
own the lower-level operation key.

Declaration lookup can succeed even when a Component is not participating.
Invocation and control then return bounded Component errors instead of
pretending a declaration-only Component is live.

## Core Instance and raw observation

`core::Instance` is the lower-level live Core realization. SDK `Instance` is
the normal high-level Fabric façade around it.

```text
Instance              = semantic lifecycle, observation, and bounded operation surface
fabric::core::Instance = lower-level runtime machinery
```

`core()` remains available for deliberate low-level work, including
`instance.core().report()`. Normal users should prefer `Instance::observe()`
and the bounded Component surface. See the [Advanced Raw API](../advanced/raw-api.md)
for direct Core authoring.

## Three identities

These identifiers refer to different things:

| Identifier | Meaning |
| --- | --- |
| `CompositionId` | Which declaration this Instance came from. |
| `MaterializationProfile` | Which occurrence-specific intent produced it. |
| `InstanceId` | The named live Instance. |
| `InstanceGeneration` | This specific materialized runtime incarnation. |

```text
CompositionId != MaterializationProfile != MaterializationPlan != InstanceId != InstanceGeneration
```

Core mints a fresh `InstanceGeneration` for every materialization. The
generation distinguishes separate runtime incarnations; it is not a
Composition, package, schema, deployment, or user-selected version.

### Example B: two Instances from one Composition

```rust
let first = composition.materialize("example.instance.first")?;
let second = composition.materialize("example.instance.second")?;

assert_ne!(first.instance_id(), second.instance_id());
assert_ne!(first.generation(), second.generation());
```

One Composition can create independent runtime state for both Instances.

### Example C: same InstanceId, fresh generation

The same logical `InstanceId` does not mean the same runtime incarnation:

```rust
let first = composition.materialize("example.instance.local")?;
let second = composition.materialize("example.instance.local")?;

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
runtime modules. Stopping uses the reverse dependency order. Otherwise
independent modules use the current deterministic flattened Block/module
insertion order as a tie-break; it is not semantic dependency priority. See
the [Composition Block clarification](composition.md#advanced-structural-machinery).

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

## Failure and interruption boundaries

Materialization can fail while constructing, context-binding, exporting, or
binding runtime modules. In those cases Fabric returns no usable `Instance`:
there is no `Ready` lifecycle to start and no report to inspect. A generation
may have been minted internally while materialization was attempted, but it is
not thereby established as a live observable Instance incarnation.

After an Instance has been returned, initialize or start failure is different:
the same generation is stopped, the operation returns `InstanceError`, and
`InstanceReport` shows the current stopped state. Reverse-order `stop()` calls
are bounded cleanup for initialized modules, not a guarantee that arbitrary
external side effects were rolled back.

Component materialization is local to a Component participation. A failed
preparation leaves no active participation or callable registered operation; a
later caller retry may create a fresh participation when the Component runtime
permits it. Dematerialization revokes future participation and calls, but does
not retroactively cancel work already executing. A Component failure does not
by itself stop the Instance.

A process crash is not `stop()`: Fabric does not persist Instance lifecycle,
generation allocation, Component participation, controls, or reports across a
process restart. Retrying an operation is a caller action; materializing again
creates a new generation. Neither is generic Fabric recovery.

## Generational declarative change

Fabric's structural boundary for a realization or declaration change is new
declarative truth materialized as a fresh Instance generation. It does not
mutate the module graph, provider selections, exports, or Host validation of a
running Instance.

For example, two independently authored Compositions may retain the same
Gateway Component and Component Config while choosing different Adapters. They
can materialize under the same `InstanceId`, receive different generations,
and run concurrently. Each generation creates its own runtime modules,
Adapter provider, resolved contracts, and Component participations. Fabric
does not reuse those runtime values across generations.

This is not a generic replacement controller. Fabric does not infer that one
Composition supersedes another, that a new generation is authoritative, or
that starting a new generation cuts over traffic or work. It also does not
infer migration, state transfer, rollback, or a safe retirement order. An
external owner chooses any such policy and may explicitly stop the older
generation when appropriate.

Compatibility remains a materialization law: a contract, Resource/System
schema, or Host requirement can establish that a selection can bind or
materialize. It does not establish that replacing an old realization is safe.
In particular, Component internal state, Resource/provider data, and System
runtime state are not generically transferred. Re-declaring the same semantic
Component Config in a new Composition is declaration continuity, not runtime
state migration.

For a Component, these are separate declaration changes rather than first-class
replacement categories:

- The same Component Config with a different Adapter changes realization.
- A different Component Config with the same Adapter changes declaration.
- Changing both changes both declaration inputs.

### Example E: inspect declaration and runtime separately

```rust
let manifest = composition.manifest();
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

Each materialization collects and retains its own selected runtime export values. Two
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
instance.stop().expect("stop Instance");
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
