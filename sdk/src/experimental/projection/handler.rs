use super::{MaterializedProjection, ProjectionError};

pub trait Projector<Request, Projection, Material>: Send + Sync {
    fn project(
        &self,
        request: &Request,
        projection: &Projection,
    ) -> Result<MaterializedProjection<Material>, ProjectionError>;
}
