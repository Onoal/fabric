#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Current aggregate runtime condition, separate from lifecycle state.
pub enum Health {
    Healthy,
    Degraded,
    Unavailable,
}
