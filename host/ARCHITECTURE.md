# Fabric Host Boundary

`fabric-host` describes environmental compatibility facts for Adapter
selection and materialization.

Host is not:

- a Resource
- a Component
- a Module
- a Contract provider
- a runtime service locator
- an authority object

Canonical law:

```text
Resource semantic compatibility
AND
Host environmental compatibility
    ->
viable Adapter candidate
```

Host requirements belong to concrete Adapter machinery. They do not change
Resource semantics and they do not participate in the running Core contract
graph.

