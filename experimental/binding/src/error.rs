use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BindingError {
    #[error("{message}")]
    InvalidInput { message: String },
}

impl BindingError {
    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }
}
