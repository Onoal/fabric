# onoal-fabric-component

`onoal-fabric-component` provides Fabric's advanced Component host and
participation model. A Component declaration owns semantic identity and may
declare Config, Relations, and an API. A realization prepares one
generation-scoped `ComponentParticipation`; invocation, host control,
readiness, communication, and surfaces are supporting integration machinery.

A Component owns behavior and its declared semantic requirements. It does not
own a Resource/System's implementation choice, global orchestration, or a
deployment topology. Normal applications should prefer
[`onoal-fabric`](https://crates.io/crates/onoal-fabric); direct use is useful
for advanced Component realization, invocation, or operator integrations.

The crate groups its public low-level surface by ownership:

- `declaration` contains runtime-free semantic metadata;
- `participation` contains live occurrence identity and diagnostics;
- `invocation` contains typed endpoint keys and provenance;
- `operator` contains the Instance-local Component host and control APIs;
- `advanced` contains handwritten realization, communication, and surface seams.

Normal `component!` authors should not need registrars, operation identities,
or host internals.

Read the [Component concept](https://github.com/Onoal/fabric/blob/main/docs/concepts/component.md)
and the [Fabric manual](https://github.com/Onoal/fabric/tree/main/docs).
