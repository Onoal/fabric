# Config

**Config** is the typed declarative creation contract a definition author
exposes to the consumer that creates an occurrence.

```text
definition creator -> consumer selecting an occurrence -> Config value
```

Resource Config configures a semantic Resource occurrence; System Config
configures a semantic System selection; Adapter Config configures a concrete
realization occurrence. The owner differs, but the authoring law is the same.

Config is not runtime state, runtime control, lifecycle, or an implementation
registry. Fabric treats the selected value as immutable input. A fresh
materialization receives a cloned declarative Config value; live machinery
belongs in runtime state instead.

## No Config

No declaration means no normal Config API or ceremony:

```rust
let store = Store::select("primary")?;
let system = SharedClock::select()?;
let adapter = MemoryStore::new();
```

An explicit legacy `config {}` remains compatible, but new normal authoring
omits it.

## Inline Config

Use inline fields for a small, direct public contract. Fabric generates the
public `StoreConfig` type.

```rust
fabric::resource! {
    Store {
        id: "example.store";
        config { namespace: String; }
        contracts { primary Api {
            id: "example.store.api";
            fn namespace(&self) -> String;
        }}
        runtime { fn namespace(&self) -> String { self.config().namespace.clone() } }
    }
}

let store = Store::select("users", StoreConfig { namespace: "users".into() })?;
```

## Creator-owned Rust types

Use an ordinary Rust type when the creator needs constructors, enums, nested
types, validation helpers, or `Default`:

```rust
enum Backend {
    Memory,
    Sqlite { path: std::path::PathBuf },
    Redis { endpoint: String, database: u32 },
}

#[derive(Clone)]
struct LocalStoreConfig { backend: Backend }

fabric::adapter! {
    LocalStore for resource Store implements StoreRealization {
        config: LocalStoreConfig;
        runtime { /* realization methods use self.config() */ }
    }
}
```

Fabric generates no wrapper around `LocalStoreConfig`. Rust types express
required fields, `Option<T>`, variants, nesting, constructors, and validation;
Fabric adds no second configuration language, Config IDs, Config versions,
serialization requirement, or dynamic map.

## Semantic and realization Config compose

A semantic Resource selection and its Adapter realization may each have Config
without collision:

```text
KeyValueStore("users") Config: namespace = users
LocalKeyValue Config: backend = Redis { endpoint, database }
```

The first describes the semantic occurrence. The second describes how the
selected realization is configured. Config neither changes semantic identity
nor participates in contract compatibility matching.

## Runtime access

Generated Resource, System, and Adapter runtime code reads Config through
`self.config()`. That accessor is available to runtime methods and lifecycle
hooks. It is read-only; live mutable state belongs in `state`/ordinary private
runtime machinery, not Config.

Return to the [Concept map](README.md).
