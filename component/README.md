# onoal-fabric-component

`onoal-fabric-component` provides Fabric's typed behavioral participation
model: Component declarations, operations, invocation, participation, and
readiness/control primitives.

A Component owns behavior and its declared semantic requirements. It does not
own a Resource/System's implementation choice, global orchestration, or a
deployment topology. Normal applications should prefer
[`onoal-fabric`](https://crates.io/crates/onoal-fabric); direct use is useful
for advanced Component runtime integrations.

Read the [Component concept](https://github.com/Onoal/fabric/blob/main/docs/concepts/component.md)
and the [Fabric manual](https://github.com/Onoal/fabric/tree/main/docs).
