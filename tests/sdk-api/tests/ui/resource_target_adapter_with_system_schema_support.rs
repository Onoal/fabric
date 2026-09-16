use fabric_core::{ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric::prelude::{
    AdapterDefinition, AdapterSystemSchemaSupport, SystemId,
};
use fabric_test_resource_clock::{Clock, ClockConfig};

#[derive(Clone)]
struct WrongSchemaSupportClockAdapter;

impl AdapterDefinition for WrongSchemaSupportClockAdapter {
    type Target = Clock;
    type Compatibility = AdapterSystemSchemaSupport;

    fn compatibility(&self) -> Self::Compatibility {
        AdapterSystemSchemaSupport::provisional(
            SystemId::new("fabric.test.system.foreign").expect("system id"),
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
    let _ = Clock::select("primary", ClockConfig::default())
        .expect("selection")
        .using(WrongSchemaSupportClockAdapter);
}
