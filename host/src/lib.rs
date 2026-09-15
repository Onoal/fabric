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
