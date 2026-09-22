# Lifecycle

Fabric lifecycle is orchestration over live graph participants. A materialized
Instance generation proceeds:

```text
Ready -> Running -> Stopped
```

`Stopped` is terminal for that generation; a fresh materialization creates a
fresh generation. Core orders materialize, bind context/dependencies,
initialize, start, and fallible stop according to the resolved dependency graph.
On failed startup it deterministically performs best-effort reverse cleanup
while retaining the initiating error and cleanup evidence.

Concrete hooks belong to the live owner. A self realization may author hooks;
an Adapter realization owns hooks for its concrete machinery. A pure derived
semantic provider need not invent stateful lifecycle work. `stop` is cleanup,
not restart, recovery, deployment, or policy.

Lifecycle is independent from [Health](health.md). Read [Instance](instance.md)
for generation semantics and [Realization](realization.md) for ownership.
