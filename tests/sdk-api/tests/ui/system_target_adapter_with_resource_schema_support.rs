use fabric_core::{ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric_sdk::prelude::{
    AdapterDefinition, AdapterResourceSchemaSupport, ResourceId,
};
use fabric_test_system_operations::{AdaptedOperations, AdaptedOperationsConfig};

#[derive(Clone)]
struct WrongSchemaSupportSystemAdapter;

impl AdapterDefinition for WrongSchemaSupportSystemAdapter {
    type Target = AdaptedOperations;
    type SchemaSupport = AdapterResourceSchemaSupport;

    fn schema_support(&self) -> Self::SchemaSupport {
        AdapterResourceSchemaSupport::provisional(
            ResourceId::new("fabric.test.resource.foreign").expect("resource id"),
        )
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        ModuleDeclaration::new(provider_module_id)
    }

    fn materialize_provider(
        &self,
        _provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>> {
        panic!("not needed for compile-fail proof")
    }
}

fn main() {
    let _ = AdaptedOperations::select(AdaptedOperationsConfig::default())
        .expect("system selection")
        .using(WrongSchemaSupportSystemAdapter);
}
