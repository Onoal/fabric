//! System identity and compatibility primitives for Fabric.
//!
//! A System is an instance-wide shared semantic capability. This crate owns
//! its typed identity and compatibility model, not Adapter implementation,
//! lifecycle orchestration, or Component behavior. Normal authors should use
//! `fabric::system!` through the `fabric` umbrella crate.

#![forbid(unsafe_code)]

mod error;
mod model;
#[cfg(test)]
mod source_guards;
#[cfg(test)]
mod tests;

pub use error::SystemCompatibilityError;
pub use model::{
    AdapterSystemSchemaSupport, SystemId, SystemSchemaCompatibilityRequirement,
    SystemSchemaDescriptor, SystemSchemaIdentity, SystemSchemaRequirement, SystemSchemaVersion,
};
