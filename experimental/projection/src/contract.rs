use std::sync::Arc;

use crate::ProjectionError;

pub trait ProjectionService<Request, Policy, Prepared>: Send + Sync {
    fn prepare(
        &self,
        requests: &[Request],
        policies: &[Policy],
    ) -> Result<Prepared, ProjectionError>;
}

pub struct ProjectionContract<Request, Policy, Prepared> {
    inner: Arc<dyn ProjectionService<Request, Policy, Prepared>>,
}

impl<Request, Policy, Prepared> Clone for ProjectionContract<Request, Policy, Prepared> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<Request, Policy, Prepared> ProjectionContract<Request, Policy, Prepared> {
    pub fn new(inner: Arc<dyn ProjectionService<Request, Policy, Prepared>>) -> Self {
        Self { inner }
    }

    pub fn prepare(
        &self,
        requests: &[Request],
        policies: &[Policy],
    ) -> Result<Prepared, ProjectionError> {
        self.inner.prepare(requests, policies)
    }
}
