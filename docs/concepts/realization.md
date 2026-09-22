# Realization

A **realization** is the concrete live machinery that makes a semantic
Resource or System operational in one materialized occurrence.

```text
semantic subject -- API --> consumer
       |
       +-- self realization
       |
       `-- Adapter realization
```

The semantic subject owns identity, semantic Config, Relations, and its API.
The realization owns implementation details: concrete Config, live state,
lifecycle hooks, and its own health observation. The same semantic target can
be selected with different compatible Adapters in different Compositions.

## Three forms

1. **Self realization**: the Resource/System explicitly owns runtime behavior.
   Use `state`, `runtime`, or `lifecycle` only when that semantic owner truly
   owns that live behavior.
2. **Direct Adapter realization**: the Adapter implements the target API
   directly. This is the normal path.
3. **Differential realization**: the semantic owner deliberately mediates one
   or more API methods; the Adapter supplies the remaining direct methods plus
   realization-only operations. Fabric composes one typed semantic provider
   from one selected effective-realization provider.

```text
Direct:        API <- Adapter
Differential:  API <- semantic assembly <- effective realization <- Adapter
```

There is still one selected provider for each Contract. Fabric does not select
providers method by method.

## Materialization

An API-only definition is valid declaration-time truth. It is not magically
materializable: without a self realization or selected compatible Adapter,
materialization returns the ordinary typed resolution/materialization failure.
This separates definition validity from live satisfiability.

Read [Adapter](adapter.md) for normal authoring, [State](state.md) and
[Lifecycle](lifecycle.md) for live ownership, and [Architecture](../architecture.md)
for the provider/binding model.
