pub trait ProjectionLease: Send {
    fn release(self: Box<Self>);
}

#[derive(Default)]
pub struct ProjectionLeases {
    inner: Vec<Box<dyn ProjectionLease>>,
}

impl ProjectionLeases {
    pub fn push<L>(&mut self, lease: L)
    where
        L: ProjectionLease + 'static,
    {
        self.inner.push(Box::new(lease));
    }

    pub fn append(&mut self, mut other: ProjectionLeases) {
        self.inner.append(&mut other.inner);
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn release_all(&mut self) {
        for lease in std::mem::take(&mut self.inner) {
            lease.release();
        }
    }
}

impl Drop for ProjectionLeases {
    fn drop(&mut self) {
        self.release_all();
    }
}
