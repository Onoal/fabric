# Relations

**Relations** describe how one Fabric definition requires a typed capability
from another definition. The first canonical relation is `requires`.

```rust
relations {
    requires {
        storage: Volume;
        clock: Clock;
    }
}
```

`Volume` and `Clock` may independently be Resources or Systems. Fabric derives
their owner-derived semantic API requirements from their definitions; authors do not
repeat the target kind or compatibility for the normal case. A versioned target
uses an exact requirement for the contract it was authored against. A broader
promise remains explicit: `storage: Volume(version = "^1");`.

## Declaration is not binding

```text
relation declaration -> contract requirement -> Composition resolution
                     -> binding -> typed runtime access
```

The relation name is author-owned. After a Composition resolves it, runtime and
lifecycle code use the same bound typed contract through `self.storage` or
`self.clock`. Before binding it is not callable; a missing provider causes
Composition/materialization failure.

Relations are not Config. Config is a creator-to-consumer declarative choice;
Relations are typed capabilities supplied through Composition. They also do not
define ownership, lifecycle ordering, transport, placement, or networking.

For example, a Postgres Resource can require a Volume and Clock, an
Observability System can require LogStore and Clock, a Redis Adapter can
require SecretStore, Network, and Clock, and a Component can require either
kind with the same canonical declaration:

```rust
fabric::component! {
    Indexer {
        id: "example.indexer";
        relations {
            requires {
                storage: Volume;
                clock: Clock;
            }
        }
    }
}
```

For Components, each relation name is a Component-local behavioral role:
`primary_store` and `cache_store` are distinct even when both target the same
Resource. Declaring a relation does not materialize a Component or attach a
runtime binding; a realization later receives its resolved typed dependencies.
Those are ordinary API capability needs, not new Fabric relation kinds.
