# State

**Runtime state** is mutable, occurrence-local machinery owned by the live
realization that needs it. It is not semantic definition Config.

```text
Config        = immutable declarative input
Runtime state = fresh mutable machinery for one materialization
```

A self-realizing Resource/System may own state when its own behavior is live.
An Adapter owns connection pools, sockets, child-process handles, caches, and
other concrete machinery for an adapted target. Handles for the same
materialized occurrence share that state; a fresh Instance generation receives
fresh state.

Do not put mutable runtime objects in Config, derive semantic identity from
state, or use state as a live reconfiguration channel. See [Config](config.md),
[Realization](realization.md), and [Lifecycle](lifecycle.md).
