#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ClockError {
    #[error("clock is unavailable: {message}")]
    Unavailable { message: String },
}
