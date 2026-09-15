use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProjectionError {
    #[error("projection unavailable")]
    Unavailable,
    #[error("{message}")]
    InvalidInput { message: String },
    #[error("{message}")]
    MaterializationFailed { message: String },
}

impl ProjectionError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    pub fn materialization_failed(message: impl Into<String>) -> Self {
        Self::MaterializationFailed {
            message: message.into(),
        }
    }
}
