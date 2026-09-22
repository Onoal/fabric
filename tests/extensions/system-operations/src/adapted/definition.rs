use fabric_system::{SystemId, SystemSchemaVersion};

fabric::system! {
    pub AdaptedOperations {
        id: "fabric.test.operations.adapted";

        version: "2.0.0";

        config {}

        api {
            fn current_marker(&self) -> crate::OperationMarker;
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for AdaptedOperationsConfig {
    fn default() -> Self {
        Self {}
    }
}

pub fn adapted_operations_system_id() -> SystemId {
    adapted_operations::raw::system_id()
}

pub fn adapted_operations_schema_version() -> SystemSchemaVersion {
    SystemSchemaVersion::parse("2.0.0").expect("static adapted operations schema version")
}
