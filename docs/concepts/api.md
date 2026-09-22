# API

An **API** is the semantic callable capability a Resource or System exposes to
its consumers. Resource and System use the same normal authoring form:

```rust
fabric::resource! {
    KeyValueStore {
        id: "example.key-value";
        api {
            fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, KeyValueError>;
            fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), KeyValueError>;
        }
    }
}

fabric::system! {
    Clock {
        id: "example.clock";

        api {
            fn now(&self) -> u64;
        }

        runtime {
            fn now(&self) -> u64 { 0 }
        }
    }
}
```

The API contains signatures only. It is not runtime implementation, transport,
an operation system, or concrete realization implementation. Fabric lowers the
single API into its typed primary contract, service trait, wrapper, and Core
contract key. Those generated details remain available for advanced and legacy
interoperation, but are not normal authoring decisions.

## Owner-derived identity and version

The primary API contract identity is derived from its semantic owner:

```text
Resource "example.key-value" -> "fabric.resource.api.example.key-value"
System   "example.clock"     -> "fabric.system.api.example.clock"
```

This is stable across Rust module placement, Config, Relations, and runtime
implementation. An owner `version: "1.2.0";` also determines the normal API
contract version. Omit it and both owner and API are provisional. API-local IDs
and versions are therefore not part of the normal language.

`api {}` is valid for an explicit marker capability. The supported method
subset remains the existing synchronous, non-generic `&self` form with simple
named arguments. `async`, generic methods, arbitrary receiver forms, and
method bodies are rejected with API-level diagnostics.

## API, Config, Relations, and Runtime

These sections answer different questions:

```text
Config    = which declarative occurrence choices are supplied?
Relations = which other semantic capabilities are required?
API       = what callable capability is exposed?
Runtime   = how does this live realization provide that capability?
```

For example, a Postgres-like Resource can use all four independently:

```rust
fabric::resource! {
    Postgres {
        id: "example.postgres";
        config { namespace: String; }
        relations { requires { storage: Volume; clock: Clock; } }
        api { fn query(&self, sql: String) -> Result<Vec<String>, QueryError>; }
        runtime {
            fn query(&self, sql: String) -> Result<Vec<String>, QueryError> {
                let _ = (&self.storage, &self.clock, &self.config().namespace, sql);
                Ok(Vec::new())
            }
        }
    }
}
```

Relations target this generated semantic API. After normal binding, runtime and
lifecycle code call it through typed fields such as `self.storage.get(...)` or
`self.clock.now()`.

For an API-only Resource/System selected with a canonical Adapter, the Adapter
provides this API directly. The semantic definition does not grow a forwarding
runtime merely because it is adapted. An explicit realization boundary remains
available for advanced semantic mediation where the lower implementation
interface differs. See [Realization](realization.md) and the
[0.5 migration guide](../migrations/0.5.md) for the legacy compatibility form.
