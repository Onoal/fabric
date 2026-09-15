use fabric_component::{ComponentError, ComponentId, ComponentRuntimeDefinition, ComponentRuntimeScope};
use fabric_core::Health;
use fabric_sdk::prelude::ComponentDefinition;

struct ForeignComponent;

impl ComponentDefinition for ForeignComponent {
    type Config = ();

    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.foreign").expect("component id")
    }

    fn prepare(
        _config: &Self::Config,
        _scope: &ComponentRuntimeScope,
    ) -> Result<Health, ComponentError> {
        Ok(Health::Healthy)
    }

    fn runtime_definition(_config: Self::Config) -> ComponentRuntimeDefinition {
        ComponentRuntimeDefinition::new(
            ComponentId::new("fabric.test.hijacked").expect("component id"),
            |_scope| Ok(Health::Healthy),
        )
    }
}

fn main() {}
