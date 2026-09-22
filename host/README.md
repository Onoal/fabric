# onoal-fabric-host

`onoal-fabric-host` defines the environmental facts used to decide whether a
concrete realization can materialize: operating system, architecture, and
named host facilities.

It owns Host descriptions and compatibility requirements. It does **not** own
semantic Resources, Systems, provider selection, scheduling, placement, or a
runtime service registry. Normal applications normally depend on
[`onoal-fabric`](https://crates.io/crates/onoal-fabric) instead; depend on this
crate directly when building a low-level realization or integration.

```rust
use fabric_host::HostDescriptor;

let host = HostDescriptor::native();
```

Read the [Fabric manual](https://github.com/Onoal/fabric/tree/main/docs) and
the [Host concept](https://github.com/Onoal/fabric/blob/main/docs/concepts/host.md)
for the boundary between Host compatibility and semantic capability.
