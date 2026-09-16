use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use fabric::prelude::*;
use third_party_augmentation_consumers::{
    ComponentXConsumer, ComponentYConsumer, ResourceConsumer, SystemConsumer,
};
use third_party_augmentation_semantics::{
    ComponentX, ComponentY, ResourceAugmentation as ResourceSemantic,
    SystemAugmentation as SystemSemantic,
};
use third_party_augmentation_support::{
    ComponentXSupport, ComponentYSupport, ResourceSupport, SystemSupport,
};
use third_party_base_semantics::{
    EchoInput, EchoOutput, ThirdPartyClock, ThirdPartyClockConfig, ThirdPartyComponent,
    ThirdPartyComponentConfig, ThirdPartyStore, ThirdPartyStoreConfig, third_party_component,
};

fn run_witness() -> Result<(), String> {
    let store = ThirdPartyStore::select("primary", ThirdPartyStoreConfig { value: 7 })
        .map_err(|error| error.to_string())?;
    let resource =
        fabric::ResourceAugmentation::<ThirdPartyStore, ResourceSemantic>::attach(&store, ())
            .map_err(|error| error.to_string())?
            .using(ResourceSupport);
    let resource_requirement = resource
        .require_from(&store)
        .map_err(|error| error.to_string())?;
    let resource_bound = Arc::new(AtomicUsize::new(0));
    let resource_consumer =
        ResourceConsumer::new(resource_requirement, Arc::clone(&resource_bound));
    let resource_selections = resource_consumer.provider_selections();

    let clock = ThirdPartyClock::select(ThirdPartyClockConfig { value: 11 })
        .map_err(|error| error.to_string())?;
    let system = fabric::SystemAugmentation::<ThirdPartyClock, SystemSemantic>::attach(&clock, ())
        .map_err(|error| error.to_string())?
        .using(SystemSupport);
    let system_requirement = system
        .require_from(&clock)
        .map_err(|error| error.to_string())?;
    let system_bound = Arc::new(AtomicUsize::new(0));
    let system_consumer = SystemConsumer::new(system_requirement, Arc::clone(&system_bound));
    let system_selections = system_consumer.provider_selections();

    let x_prepared = Arc::new(AtomicUsize::new(0));
    let y_prepared = Arc::new(AtomicUsize::new(0));
    let x = ThirdPartyComponent::define(ThirdPartyComponentConfig {})
        .augment::<ComponentX>(())
        .map_err(|error| error.to_string())?
        .using(ComponentXSupport(Arc::clone(&x_prepared)));
    let x_requirement = x.requirement();
    let y = x
        .into_set()
        .augment::<ComponentY>(())
        .map_err(|error| error.to_string())?
        .using(ComponentYSupport(Arc::clone(&y_prepared)));
    let y_requirement = y.requirement();
    let x_bound = Arc::new(AtomicUsize::new(0));
    let y_bound = Arc::new(AtomicUsize::new(0));
    let x_consumer = ComponentXConsumer::new(x_requirement, Arc::clone(&x_bound));
    let y_consumer = ComponentYConsumer::new(y_requirement, Arc::clone(&y_bound));
    let x_selection = x_consumer.provider_selection();
    let y_selection = y_consumer.provider_selection();

    let built = Fabric::new("third.party.augmentation.app")
        .map_err(|error| error.to_string())?
        .resource(store)
        .resource_augmentation(resource)
        .system(clock)
        .system_augmentation(system)
        .component(y)
        .block("resource-consumer", |block| block.module(resource_consumer))
        .map_err(|error| error.to_string())?
        .block("system-consumer", |block| block.module(system_consumer))
        .map_err(|error| error.to_string())?
        .block("component-x-consumer", |block| block.module(x_consumer))
        .map_err(|error| error.to_string())?
        .block("component-y-consumer", |block| block.module(y_consumer))
        .map_err(|error| error.to_string())?
        .select_provider(resource_selections[0].clone())
        .select_provider(resource_selections[1].clone())
        .select_provider(system_selections[0].clone())
        .select_provider(system_selections[1].clone())
        .select_provider(x_selection)
        .select_provider(y_selection)
        .build()
        .map_err(|error| error.to_string())?;
    assert_eq!(built.manifest().resource_augmentations().len(), 1);
    assert_eq!(built.manifest().system_augmentations().len(), 1);
    assert_eq!(built.manifest().component_augmentations().len(), 2);

    let mut instance = built
        .materialize_named("third-party-witness")
        .map_err(|error| error.to_string())?;
    instance.start().map_err(|error| error.to_string())?;
    assert_eq!(resource_bound.load(Ordering::SeqCst), 1);
    assert_eq!(system_bound.load(Ordering::SeqCst), 1);
    assert_eq!(x_bound.load(Ordering::SeqCst), 1);
    assert_eq!(y_bound.load(Ordering::SeqCst), 1);
    let components = instance
        .components()
        .ok_or_else(|| "component runtime is unavailable".to_owned())?;
    components
        .materialize::<ThirdPartyComponent>()
        .map_err(|error| error.to_string())?;
    assert_eq!(x_prepared.load(Ordering::SeqCst), 1);
    assert_eq!(y_prepared.load(Ordering::SeqCst), 1);
    let output: EchoOutput = futures::executor::block_on(
        components.invoke_external(&third_party_component::operations::echo(), EchoInput),
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(output.0, "base");
    Ok(())
}

fn main() {
    run_witness().expect("third-party augmentation witness");
    println!("third-party augmentation equality verified");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_resource_compiles_and_builds_without_augmentation_dependencies() {
        let store = ThirdPartyStore::select("base-only", ThirdPartyStoreConfig { value: 1 })
            .expect("base resource selection");
        Fabric::new("third.party.base.only")
            .expect("fabric")
            .resource(store)
            .build()
            .expect("base-only composition");
    }

    #[test]
    fn independently_owned_augmentations_bind_through_public_fabric() {
        run_witness().expect("witness");
    }
}
