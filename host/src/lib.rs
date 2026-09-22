//! Environmental compatibility primitives for Fabric realizations.
//!
//! A [`HostDescriptor`] describes materialization facts such as the operating
//! system, architecture, and available named facilities. [`HostRequirement`]
//! states what a concrete realization needs. Host is not a semantic Resource
//! or System, and it does not select or schedule a provider.
//!
//! Normal applications generally use the `fabric` umbrella crate. This crate
//! is the focused public dependency for advanced realization integrations.

#![forbid(unsafe_code)]

mod descriptor;
mod error;
mod identifier;
mod requirement;

#[cfg(test)]
mod source_guards;
#[cfg(test)]
mod tests;

pub use descriptor::HostDescriptor;
pub use error::{HostCompatibilityError, HostError};
pub use identifier::{HostArchitecture, HostFacilityId, HostOperatingSystem};
pub use requirement::HostRequirement;
