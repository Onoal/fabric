use std::sync::Arc;

use fabric_component::{
    Component, ComponentId, ComponentRuntime, ComponentRuntimeService, ComponentRuntimeStatus,
    Surface, SurfaceId,
};
use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionError, CompositionId,
    ContractRequirement, Health, InstanceId, ModuleBindings, ModuleContract, ModuleError, ModuleId,
    ModuleRuntime,
};

#[derive(Clone)]
struct StaticComponentRuntimeService {
    instance_id: InstanceId,
}

impl ComponentRuntimeService for StaticComponentRuntimeService {
    fn instance_id(&self) -> InstanceId {
        self.instance_id.clone()
    }

    fn current_instance_id(&self) -> Result<InstanceId, fabric_component::ComponentError> {
        Ok(self.instance_id())
    }

    fn current_status(&self) -> ComponentRuntimeStatus {
        ComponentRuntimeStatus::new(
            self.instance_id(),
            None,
            fabric_component::ComponentRuntimeLifecycle::Stopped,
            Health::Unavailable,
        )
    }

    fn current_lifecycle(&self) -> fabric_component::ComponentRuntimeLifecycle {
        fabric_component::ComponentRuntimeLifecycle::Ready
    }

    fn current_health(&self) -> Health {
        Health::Healthy
    }
}

#[derive(Clone)]
struct SurfaceParticipantModule {
    module_id: ModuleId,
    component_id: ComponentId,
    runtime_requirement: ContractRequirement<ComponentRuntime>,
    surface_id: SurfaceId,
}

impl SurfaceParticipantModule {
    fn new(module_id: &str, component_id: &str, surface_id: &str) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            component_id: ComponentId::new(component_id).expect("component id"),
            runtime_requirement: ContractRequirement::provisional(
                fabric_component::component_runtime_contract_id(),
            ),
            surface_id: SurfaceId::new(surface_id).expect("surface id"),
        }
    }
}

impl ModuleRuntime for SurfaceParticipantModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        Vec::new()
            .into_iter()
            .map(fabric_core::ProvidedContractDeclaration::provisional)
            .collect()
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.runtime_requirement.id().clone()]
            .into_iter()
            .map(fabric_core::ContractRequirementDeclaration::provisional)
            .collect()
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let runtime = bindings
            .resolve(&self.runtime_requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        let component = Component::bind(self.component_id.clone(), runtime.as_ref());
        let _surface = Surface::new(component, self.surface_id.clone());
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}

fn static_component_runtime(instance_id: &str) -> ComponentRuntime {
    let instance_id = InstanceId::new(instance_id).expect("instance id");
    ComponentRuntime::new(Arc::new(StaticComponentRuntimeService { instance_id }))
}

#[test]
fn surface_requires_component_runtime_membership() {
    let block = BlockBuilder::new(BlockId::new("component.surface.block").expect("block id"))
        .register_module(SurfaceParticipantModule::new(
            "component.surface.participant",
            "photos",
            "photos.main",
        ))
        .build();
    let error = CompositionBuilder::new(
        CompositionId::new("component.surface.composition").expect("composition id"),
    )
    .register_block(block)
    .build()
    .expect_err("surface participation without component runtime must fail");

    assert!(matches!(
        error,
        CompositionError::MissingProvider {
            contract_id,
            ..
        } if contract_id == fabric_component::component_runtime_contract_id()
    ));
}

#[test]
fn stable_surface_identity_is_preserved() {
    let runtime = static_component_runtime("component.main");
    let owner = Component::bind(ComponentId::new("photos").expect("component id"), &runtime);

    let left = Surface::new(
        owner.clone(),
        SurfaceId::new("photos.main").expect("surface id"),
    );
    let right = Surface::new(owner, SurfaceId::new("photos.main").expect("surface id"));

    assert_eq!(left, right);
}
