# onoal-fabric-resource

`onoal-fabric-resource` contains the identity, occurrence, compatibility, and
selection primitives for Fabric Resources. A Resource is an occurrence-based
semantic capability; it is not automatically its concrete implementation.

This crate owns Resource identity and selection semantics. It does **not** own
Adapter machinery, live runtime state, lifecycle policy, Components, or Host
placement. Normal authors should use
[`onoal-fabric`](https://crates.io/crates/onoal-fabric) and `resource!`; direct
use is for advanced integrations that need the lower typed model.

See the [Resource concept](https://github.com/Onoal/fabric/blob/main/docs/concepts/resource.md)
and the [Fabric manual](https://github.com/Onoal/fabric/tree/main/docs).
