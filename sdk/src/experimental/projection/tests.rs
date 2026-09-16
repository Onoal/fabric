use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{
    MaterializedProjection, ProjectionContract, ProjectionError, ProjectionLease, ProjectionLeases,
    ProjectionService, Projector,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Request(&'static str);
#[derive(Clone, Debug, PartialEq, Eq)]
struct Policy(&'static str);
#[derive(Clone, Debug, PartialEq, Eq)]
struct Material(&'static str);

struct TestProjector;

impl Projector<Request, Policy, Material> for TestProjector {
    fn project(
        &self,
        request: &Request,
        policy: &Policy,
    ) -> Result<MaterializedProjection<Material>, ProjectionError> {
        Ok(MaterializedProjection::new(Material(
            if request.0 == policy.0 {
                policy.0
            } else {
                return Err(ProjectionError::invalid_input("mismatch"));
            },
        )))
    }
}

struct TestService;

impl ProjectionService<Request, Policy, Material> for TestService {
    fn prepare(
        &self,
        requests: &[Request],
        policies: &[Policy],
    ) -> Result<Material, ProjectionError> {
        TestProjector
            .project(&requests[0], &policies[0])
            .map(|value| value.into_parts().0)
    }
}

struct CountingLease(Arc<AtomicUsize>);

impl ProjectionLease for CountingLease {
    fn release(self: Box<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn projection_contract_and_leases_remain_usable() {
    let contract = ProjectionContract::new(Arc::new(TestService));
    assert_eq!(
        contract.prepare(&[Request("queue")], &[Policy("queue")]),
        Ok(Material("queue"))
    );

    let released = Arc::new(AtomicUsize::new(0));
    let mut leases = ProjectionLeases::default();
    leases.push(CountingLease(Arc::clone(&released)));
    drop(MaterializedProjection::with_leases(
        Material("leased"),
        leases,
    ));
    assert_eq!(released.load(Ordering::SeqCst), 1);
}
