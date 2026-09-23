use fabric_component::{ComponentError, ComponentId, ComponentParticipationRealization, ComponentParticipationScope};
use fabric_core::Health;
use fabric::authoring::component::ComponentDefinition;

struct ForeignComponent;

impl ComponentDefinition for ForeignComponent {
    type Config = ();

    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.foreign").expect("component id")
    }

    fn prepare(
        _config: &Self::Config,
        _scope: &ComponentParticipationScope,
    ) -> Result<Health, ComponentError> {
        Ok(Health::Healthy)
    }

    fn runtime_definition(_config: Self::Config) -> ComponentParticipationRealization {
        ComponentParticipationRealization::new(
            ComponentId::new("fabric.test.hijacked").expect("component id"),
            |_scope| Ok(Health::Healthy),
        )
    }
}

fn main() {}
