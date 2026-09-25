# onoal-fabric-sdk-macros

`onoal-fabric-sdk-macros` implements the `resource!`, `system!`, `adapter!`,
and `component!` authoring macros used by Fabric's Rust SDK.

Normal applications should depend on
[`onoal-fabric`](https://crates.io/crates/onoal-fabric), which re-exports these
macros. A direct dependency is mainly appropriate for tooling, macro-focused
testing, or advanced SDK integration. Generated raw types and helper modules
are implementation machinery, not the normal Fabric vocabulary.

Start with the [Fabric manual](https://github.com/Onoal/fabric/tree/main/docs)
rather than this crate's generated expansion details.
