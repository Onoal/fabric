# Health

**Health** is an observation of a live owner's ability to fulfill its
responsibility. It is not a lifecycle state.

```text
Lifecycle: Ready -> Running -> Stopped
Health:    Healthy | Degraded | Unavailable
```

A running Adapter can be Degraded or Unavailable without changing its lifecycle.
Likewise, Adapter/backend health is not automatically Resource/System semantic
health or whole-Instance health. Fabric preserves those boundaries rather than
inventing an implicit aggregation policy.

Health belongs with the live realization that can observe the concrete
machinery. See [Lifecycle](lifecycle.md) and [Realization](realization.md).
