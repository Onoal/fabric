#![forbid(unsafe_code)]

mod error;
mod model;
#[cfg(test)]
mod source_guards;
#[cfg(test)]
mod tests;

pub use error::BindingError;
pub use model::{
    BindingConsumer, BindingConsumerId, BindingConsumerKind, BindingId, BindingName,
    BindingTargetId, BindingTargetKind,
};
