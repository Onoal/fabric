//! Experimental projection helpers shipped with `onoal-fabric`.
//!
//! Projection is not a Core primitive, a Composition participant, or a
//! Manifest semantic category. Its API may change or disappear in a future
//! minor release, while remaining compatibility-sensitive within a patch line.

mod contract;
mod error;
mod handler;
mod model;

#[cfg(test)]
mod source_guards;
#[cfg(test)]
mod tests;

pub use contract::{ProjectionContract, ProjectionService};
pub use error::ProjectionError;
pub use handler::Projector;
pub use model::{MaterializedProjection, ProjectionLease, ProjectionLeases};
