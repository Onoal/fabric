use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    MaterializedProjection, ProjectionContract, ProjectionError, ProjectionLease, ProjectionLeases,
    ProjectionService, Projector,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct SyntheticProjectionRequest {
    binding_name: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SyntheticProjectionPolicy {
    binding_name: &'static str,
    material: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SyntheticProjectionMaterial(&'static str);

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct SyntheticPreparedProjection {
    materials: Vec<(&'static str, SyntheticProjectionMaterial)>,
}

struct SyntheticProjector;

impl Projector<SyntheticProjectionRequest, SyntheticProjectionPolicy, SyntheticProjectionMaterial>
    for SyntheticProjector
{
    fn project(
        &self,
        request: &SyntheticProjectionRequest,
        projection: &SyntheticProjectionPolicy,
    ) -> Result<MaterializedProjection<SyntheticProjectionMaterial>, ProjectionError> {
        if request.binding_name != projection.binding_name {
            return Err(ProjectionError::invalid_input(format!(
                "projection {} does not match request {}",
                projection.binding_name, request.binding_name
            )));
        }

        Ok(MaterializedProjection::new(SyntheticProjectionMaterial(
            projection.material,
        )))
    }
}

struct SyntheticProjectionCoordinator {
    projector: SyntheticProjector,
}

impl
    ProjectionService<
        SyntheticProjectionRequest,
        SyntheticProjectionPolicy,
        SyntheticPreparedProjection,
    > for SyntheticProjectionCoordinator
{
    fn prepare(
        &self,
        requests: &[SyntheticProjectionRequest],
        policies: &[SyntheticProjectionPolicy],
    ) -> Result<SyntheticPreparedProjection, ProjectionError> {
        let mut prepared = SyntheticPreparedProjection::default();
        for policy in policies {
            let request = requests
                .iter()
                .find(|request| request.binding_name == policy.binding_name)
                .ok_or_else(|| {
                    ProjectionError::invalid_input(format!(
                        "missing request for {}",
                        policy.binding_name
                    ))
                })?;
            let materialized = self.projector.project(request, policy)?;
            prepared
                .materials
                .push((request.binding_name, materialized.material().clone()));
        }
        Ok(prepared)
    }
}

struct CountingLease {
    released: Arc<AtomicUsize>,
}

impl ProjectionLease for CountingLease {
    fn release(self: Box<Self>) {
        self.released.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn synthetic_projection_contract_uses_the_actual_shared_rail_without_catalog_edits() {
    let contract = ProjectionContract::new(Arc::new(SyntheticProjectionCoordinator {
        projector: SyntheticProjector,
    }));

    let prepared = contract
        .prepare(
            &[SyntheticProjectionRequest {
                binding_name: "queue",
            }],
            &[SyntheticProjectionPolicy {
                binding_name: "queue",
                material: "synthetic-material",
            }],
        )
        .expect("prepare synthetic projections");

    assert_eq!(
        prepared,
        SyntheticPreparedProjection {
            materials: vec![("queue", SyntheticProjectionMaterial("synthetic-material"))],
        }
    );
}

#[test]
fn materialized_projection_leases_release_on_drop() {
    let released = Arc::new(AtomicUsize::new(0));
    let mut leases = ProjectionLeases::default();
    leases.push(CountingLease {
        released: Arc::clone(&released),
    });

    let materialized =
        MaterializedProjection::with_leases(SyntheticProjectionMaterial("leased"), leases);
    let (_material, leases) = materialized.into_parts();

    assert_eq!(released.load(Ordering::SeqCst), 0);
    drop(leases);
    assert_eq!(released.load(Ordering::SeqCst), 1);
}
