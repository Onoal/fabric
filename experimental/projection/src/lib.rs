#![forbid(unsafe_code)]

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
