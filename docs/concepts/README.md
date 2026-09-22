# Fabric Concepts

Fabric describes a system before it runs and then materializes that description
into live runtime state. The concepts below are the normal map for reading and
using Fabric.

## The concepts

**[Composition](composition.md)** is the reusable declaration of a system: its
participants, requirements, selections, and declared structure. Start here to
understand the difference between what Fabric declares and what it runs.

**[Instance](instance.md)** is one live materialization of a Composition. It
owns lifecycle, generation, and runtime state.

**[Component](component.md)** expresses semantic behavior as typed Operations.
A Component can require Resources and Systems.

**[Resource](resource.md)** is an occurrence-based technical capability. A Composition may
contain distinct occurrences of the same Resource type.

**[System](system.md)** is an instance-wide shared capability used by the
declared system.

**[Adapter](adapter.md)** realizes an adaptable Component, Resource, or System
through an explicit public realization interface.

**[Config](config.md)** is the typed creator-to-consumer declaration input for
Resource, System, and Adapter occurrences.

**[API](api.md)** is the semantic callable capability a Resource or System
exposes to its consumers.

**[Augmentation](augmentation.md)** is independently owned semantic meaning
attached to a selected Resource, System, or Component without changing the
base definition.

**Host** describes environmental compatibility needed to materialize an Adapter
realization.

**Manifest** is immutable semantic inspection of a Composition. Its diagnostics
view exposes raw/Core backing details when advanced debugging needs them.

## Relations

```text
Component --requires--> Resource
Component --requires--> System

Adapter --realizes--> Component, Resource, or System
Host --constrains compatibility of--> Adapter realization

External semantic X --augments--> Resource, System, or Component target
Support --realizes--> X for that target

Composition --declares and selects--> Components, Resources, Systems, Adapters, Host truth
Instance --materializes--> Composition
Manifest --inspects semantic declarations of--> Composition
```

Continue with [Architecture](../architecture.md) for the complete model, or
with the [Advanced Raw API](../advanced/raw-api.md) when building directly on
Core.

**[Relations](relations.md)** declare typed capabilities a Resource, System, or
Adapter requires through Composition.
