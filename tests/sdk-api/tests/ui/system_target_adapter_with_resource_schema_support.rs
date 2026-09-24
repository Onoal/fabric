use fabric::{ResourceId, authoring::AdapterDefinition, resource::AdapterResourceSchemaSupport};
use fabric_core::{ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric_test_system_operations::{AdaptedOperations, AdaptedOperationsConfig};

#[derive(Clone)]
struct WrongSchemaSupportSystemAdapter;

impl AdapterDefinition for WrongSchemaSupportSystemAdapter {
    fn adapter_definition_id(&self) -> fabric::authoring::AdapterDefinitionId {
        fabric::authoring::AdapterDefinitionId::new("test.manual.wrong-schema-support-system-adapter")
            .expect("static test Adapter definition ID is valid")
    }

    type Target = AdaptedOperations;
    type Compatibility = AdapterResourceSchemaSupport;

    fn compatibility(&self) -> Self::Compatibility {
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
