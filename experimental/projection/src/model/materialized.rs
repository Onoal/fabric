use crate::ProjectionLeases;

pub struct MaterializedProjection<Material> {
    material: Material,
    leases: ProjectionLeases,
}

impl<Material> MaterializedProjection<Material> {
    pub fn new(material: Material) -> Self {
        Self {
            material,
            leases: ProjectionLeases::default(),
        }
    }

    pub fn with_leases(material: Material, leases: ProjectionLeases) -> Self {
        Self { material, leases }
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn into_parts(self) -> (Material, ProjectionLeases) {
        (self.material, self.leases)
    }
}
