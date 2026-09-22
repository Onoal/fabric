# Host

A **Host** describes environmental facts used to decide whether a concrete
realization can materialize: operating system, architecture, and named
facilities.

Host requirements belong to Adapter/concrete-realization compatibility. They
do not change a Resource/System's semantic identity, API, Relations, or
consumer requirements.

```text
semantic target compatibility  +  Host compatibility
                 -> viable concrete realization
```

Host is not a Resource, System, Component, deployment planner, scheduler, or
runtime service locator. Read [Adapter](adapter.md) and the
[Host crate architecture](../../host/ARCHITECTURE.md) for details.
