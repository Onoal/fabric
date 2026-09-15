use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use fabric_core::{Health, InstanceGeneration, InstanceId};

use super::*;

pub(super) struct RegisteredOperation {
    pub(super) owner: ComponentParticipation,
    pub(super) definition: OperationDefinition,
    pub(super) input_type: TypeId,
    pub(super) output_type: TypeId,
    pub(super) handler: RegisteredOperationHandler,
}

type ComponentCatalog = (
    BTreeMap<ComponentId, ComponentDeclaration>,
    BTreeMap<ComponentId, ComponentRuntimeDefinition>,
);

pub(super) fn current_participation(
    state: &ComponentModuleState,
    participation: &ComponentParticipation,
) -> bool {
    state
        .components
        .get(participation.component().component_id())
        .is_some_and(|status| status.participation() == participation)
}

pub(super) fn validate_context_runtime(
    state: &ComponentModuleState,
    context: &crate::InvocationContext,
) -> Result<(), ComponentError> {
    let instance_id = state.instance_id();
    if context.instance_id() != &instance_id {
        return Err(ComponentError::InvocationContextInstanceMismatch {
            context_instance_id: context.instance_id().clone(),
            instance_id,
        });
    }
    let generation = state.current_generation()?;
    if context.generation() != generation {
        return Err(ComponentError::InvocationContextGenerationMismatch {
            context_generation: context.generation(),
            generation,
        });
    }
    Ok(())
}

fn current_participation_status<'a>(
    state: &'a ComponentModuleState,
    participation: &ComponentParticipation,
) -> Option<&'a ComponentStatus> {
    state
        .components
        .get(participation.component().component_id())
        .filter(|status| status.participation() == participation)
}

pub(super) fn current_preparing_participation(
    state: &ComponentModuleState,
    participation: &ComponentParticipation,
) -> bool {
    current_participation_status(state, participation)
        .is_some_and(|status| status.state() == ParticipationState::Preparing)
}

pub(super) fn current_active_participation(
    state: &ComponentModuleState,
    participation: &ComponentParticipation,
) -> bool {
    current_participation_status(state, participation)
        .is_some_and(|status| status.state() == ParticipationState::Active)
}

pub(super) fn participation_is_available(
    state: &ComponentModuleState,
    participation: &ComponentParticipation,
) -> bool {
    current_active_participation(state, participation)
        && effective_health(state, participation.component(), &mut BTreeSet::new()).is_available()
}

pub(super) fn effective_availability_from_health(
    health: ComponentEffectiveHealth,
) -> ComponentEffectiveAvailability {
    match health {
        ComponentEffectiveHealth::Healthy | ComponentEffectiveHealth::Degraded(_) => {
            ComponentEffectiveAvailability::Available
        }
        ComponentEffectiveHealth::Unavailable(availability) => availability,
    }
}

pub(super) fn bound_component(
    state: &ComponentModuleState,
    component_id: &crate::ComponentId,
) -> Component {
    state
        .components
        .get(component_id)
        .map(|status| status.component().clone())
        .or_else(|| {
            state
                .requirements
                .iter()
                .find(|requirement| requirement.consumer().component_id() == component_id)
                .map(|requirement| requirement.consumer().clone())
        })
        .or_else(|| {
            state
                .requirements
                .iter()
                .find(|requirement| requirement.provider().component_id() == component_id)
                .map(|requirement| requirement.provider().clone())
        })
        .unwrap_or_else(|| Component::for_instance(component_id.clone(), state.instance_id()))
}

pub(super) fn effective_health(
    state: &ComponentModuleState,
    component: &Component,
    visited: &mut BTreeSet<crate::ComponentId>,
) -> ComponentEffectiveHealth {
    if !visited.insert(component.component_id().clone()) {
        return ComponentEffectiveHealth::Unavailable(
            ComponentEffectiveAvailability::MalformedCycle {
                component: component.clone(),
            },
        );
    }

    let effective = match state.components.get(component.component_id()) {
        None => {
            ComponentEffectiveHealth::Unavailable(ComponentEffectiveAvailability::NotParticipating)
        }
        Some(status) if status.state() != ParticipationState::Active => {
            ComponentEffectiveHealth::Unavailable(ComponentEffectiveAvailability::NotActive {
                component: status.component().clone(),
                participation: status.participation().clone(),
                state: status.state(),
            })
        }
        Some(status) if status.health() == Health::Unavailable => {
            ComponentEffectiveHealth::Unavailable(
                ComponentEffectiveAvailability::IntrinsicUnavailable {
                    component: status.component().clone(),
                    health: status.health(),
                },
            )
        }
        Some(status) => {
            let mut degraded = (status.health() == Health::Degraded).then(|| {
                ComponentEffectiveHealth::Degraded(DegradedComponentHealth::Intrinsic {
                    component: status.component().clone(),
                    health: status.health(),
                })
            });
            for requirement in state.requirements.iter().filter(|requirement| {
                requirement.consumer() == status.component()
                    && requirement.kind() == ComponentRequirementKind::Required
            }) {
                let provider_health = effective_health(state, requirement.provider(), visited);
                match provider_health {
                    ComponentEffectiveHealth::Healthy => {}
                    ComponentEffectiveHealth::Degraded(_) => {
                        degraded = Some(ComponentEffectiveHealth::Degraded(
                            DegradedComponentHealth::RequiredDependencyDegraded(
                                ComponentDependencyHealthBlocker::new(
                                    requirement.contract_id().clone(),
                                    requirement.provider().clone(),
                                    provider_health,
                                ),
                            ),
                        ));
                    }
                    ComponentEffectiveHealth::Unavailable(_) => {
                        visited.remove(component.component_id());
                        return ComponentEffectiveHealth::Unavailable(
                            ComponentEffectiveAvailability::RequiredDependencyUnavailable(
                                ComponentDependencyAvailabilityBlocker::new(
                                    requirement.contract_id().clone(),
                                    requirement.provider().clone(),
                                    effective_availability_from_health(provider_health),
                                ),
                            ),
                        );
                    }
                }
            }
            degraded.unwrap_or(ComponentEffectiveHealth::Healthy)
        }
    };

    visited.remove(component.component_id());
    effective
}

pub(super) fn project_aggregate_readiness(
    state: &ComponentModuleState,
) -> ComponentAggregateReadiness {
    let mut blockers = Vec::new();
    let mut aggregate_health = Health::Healthy;
    for component_id in state.readiness_policy.required_components() {
        let component = bound_component(state, component_id);
        let effective = effective_health(state, &component, &mut BTreeSet::new());
        match &effective {
            ComponentEffectiveHealth::Healthy => {}
            ComponentEffectiveHealth::Degraded(_) => {
                aggregate_health = Health::Degraded;
                blockers.push(ComponentAggregateBlocker::RequiredComponentDegraded {
                    component_id: component_id.clone(),
                    effective_health: effective.clone(),
                });
            }
            ComponentEffectiveHealth::Unavailable(
                ComponentEffectiveAvailability::NotParticipating,
            ) => {
                aggregate_health = Health::Unavailable;
                blockers.push(
                    ComponentAggregateBlocker::RequiredComponentNotParticipating {
                        component_id: component_id.clone(),
                    },
                );
            }
            ComponentEffectiveHealth::Unavailable(_) => {
                aggregate_health = Health::Unavailable;
                blockers.push(ComponentAggregateBlocker::RequiredComponentUnavailable {
                    component_id: component_id.clone(),
                    effective_health: effective.clone(),
                });
            }
        }
    }

    let lifecycle = if aggregate_health == Health::Healthy {
        ComponentRuntimeLifecycle::Ready
    } else {
        ComponentRuntimeLifecycle::Degraded
    };

    ComponentAggregateReadiness::new(
        ComponentRuntimeStatus::new(
            state.instance_id(),
            state.status.generation(),
            lifecycle,
            aggregate_health,
        ),
        state.readiness_policy.clone(),
        blockers,
    )
}

pub(super) fn refresh_operational_status(state: &mut ComponentModuleState) {
    if !matches!(
        state.status.lifecycle(),
        ComponentRuntimeLifecycle::Ready | ComponentRuntimeLifecycle::Degraded
    ) {
        return;
    }
    state.status = project_aggregate_readiness(state).status().clone();
}

pub(super) fn validated_catalog(
    declarations: impl IntoIterator<Item = ComponentDeclaration>,
    attachments: impl IntoIterator<Item = ComponentRuntimeDefinition>,
) -> Result<ComponentCatalog, ComponentError> {
    let mut catalog = BTreeMap::new();
    for declaration in declarations {
        let component_id = declaration.component_id().clone();
        if catalog.insert(component_id.clone(), declaration).is_some() {
            return Err(ComponentError::DuplicateComponentId(component_id));
        }
    }
    let mut realized = BTreeMap::new();
    for attachment in attachments {
        let component_id = attachment.component_id().clone();
        if !catalog.contains_key(&component_id) {
            return Err(ComponentError::UnknownComponent(component_id));
        }
        if realized.insert(component_id.clone(), attachment).is_some() {
            return Err(ComponentError::DuplicateComponentRuntimeDefinition(
                component_id,
            ));
        }
    }
    Ok((catalog, realized))
}

fn controls_from_snapshot(
    control_snapshot: &ComponentControlSnapshot,
    instance_id: &InstanceId,
) -> Result<BTreeMap<ComponentId, ComponentControl>, ComponentError> {
    if control_snapshot.instance_id() != instance_id {
        return Err(ComponentError::ComponentControlSnapshotInstanceMismatch {
            snapshot_instance_id: control_snapshot.instance_id().clone(),
            instance_id: instance_id.clone(),
        });
    }

    let mut controls = BTreeMap::new();
    for entry in control_snapshot.entries() {
        let component = Component::for_instance(entry.component_id().clone(), instance_id.clone());
        controls.insert(
            entry.component_id().clone(),
            ComponentControl::new(component, entry.desired()),
        );
    }
    Ok(controls)
}

pub(super) fn validated_control_snapshot(
    control_snapshot: Option<ComponentControlSnapshot>,
) -> Result<Option<ComponentControlSnapshot>, ComponentError> {
    let Some(control_snapshot) = control_snapshot else {
        return Ok(None);
    };

    Ok(Some(control_snapshot))
}

impl Clone for RegisteredOperation {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            definition: self.definition.clone(),
            input_type: self.input_type,
            output_type: self.output_type,
            handler: Arc::clone(&self.handler),
        }
    }
}

impl ComponentModuleState {
    pub(super) fn instance_id(&self) -> InstanceId {
        self.status.instance_id().clone()
    }

    pub(super) fn current_status(&self) -> ComponentRuntimeStatus {
        match self.status.lifecycle() {
            ComponentRuntimeLifecycle::Ready | ComponentRuntimeLifecycle::Degraded => {
                project_aggregate_readiness(self).status().clone()
            }
            ComponentRuntimeLifecycle::Starting
            | ComponentRuntimeLifecycle::Stopping
            | ComponentRuntimeLifecycle::Stopped => self.status.clone(),
        }
    }

    pub(super) fn current_generation(&self) -> Result<InstanceGeneration, ComponentError> {
        self.status
            .generation()
            .ok_or(ComponentError::InstanceGenerationUnbound)
    }

    pub(super) fn transition_to(
        &mut self,
        next: ComponentRuntimeLifecycle,
    ) -> Result<(), ComponentError> {
        self.status = self.status.transition_to(next)?;
        Ok(())
    }

    pub(super) fn set_health(&mut self, health: Health) {
        self.status = self.status.with_health(health);
    }

    pub(super) fn rebind_instance_context(
        &mut self,
        instance_id: InstanceId,
        generation: InstanceGeneration,
    ) -> Result<(), ComponentError> {
        self.status = self.status.bind_runtime(instance_id.clone(), generation);
        self.component_controls = match &self.control_snapshot {
            Some(control_snapshot) => controls_from_snapshot(control_snapshot, &instance_id)?,
            None => self
                .component_controls
                .values()
                .map(|control| {
                    (
                        control.component().component_id().clone(),
                        ComponentControl::new(
                            Component::for_instance(
                                control.component().component_id().clone(),
                                instance_id.clone(),
                            ),
                            control.desired(),
                        ),
                    )
                })
                .collect(),
        };
        Ok(())
    }
}
