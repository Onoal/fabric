#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleState {
    Ready,
    Running,
    Stopped,
}
