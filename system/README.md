# onoal-fabric-system

`onoal-fabric-system` contains the identity and compatibility primitives for
Fabric Systems. A System is an instance-wide shared semantic capability. It is
not a scheduler, deployment unit, or concrete provider by itself.

This crate owns System identity and schema compatibility. It does **not** own
Resource occurrences, Adapter implementation, Component behavior, or runtime
lifecycle policy. Normal authors should prefer
[`onoal-fabric`](https://crates.io/crates/onoal-fabric) and `system!`; direct
dependency is for advanced typed integrations.

Read the [System concept](https://github.com/Onoal/fabric/blob/main/docs/concepts/system.md)
and the [Fabric manual](https://github.com/Onoal/fabric/tree/main/docs).
