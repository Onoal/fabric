#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ResourceError {
    #[error("resource input is invalid: {message}")]
    InvalidInput { message: String },
    #[error("resource preparation failed: {message}")]
    PrepareFailed { message: String },
    #[error("resource integrity failure: {message}")]
    Integrity { message: String },
}
