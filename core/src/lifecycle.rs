#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// The deliberately narrow lifecycle of one materialized Core Instance.
///
/// `Stopped` is terminal for that generation. Health and module execution
/// phases are separate concerns and do not add lifecycle variants here.
pub enum LifecycleState {
    Ready,
    Running,
    Stopped,
}
