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
