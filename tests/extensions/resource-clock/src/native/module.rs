use std::sync::{Arc, Mutex};

use fabric::authoring::AdaptableResourceDefinition;
use fabric_core::{
    ContractRequirement, Health, ModuleBindings, ModuleContract, ModuleError, ModuleId,
    ModuleRuntime, ResolvedContract,
};

use crate::{
    BoundClockRealization, ClockConfig, ClockContract, ClockLifecycleCapture,
    ClockRealizationCapture, ClockRealizationContract, ClockService, ClockTick, clock_contract_key,
};

#[derive(Clone)]
struct RealizedClock {
    state: Arc<ClockRuntimeState>,
}

#[derive(Clone)]
pub struct NativeClock {
    module_id: ModuleId,
    realization_requirement: ContractRequirement<ClockRealizationContract>,
    state: Arc<ClockRuntimeState>,
    lifecycle_capture: Option<ClockLifecycleCapture>,
    realization_capture: Option<ClockRealizationCapture>,
}

#[derive(Default)]
struct ClockRuntimeState {
    realization: Mutex<Option<Arc<ClockRealizationContract>>>,
}

impl NativeClock {
    pub fn new(module_id: &str, config: ClockConfig) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            realization_requirement: crate::Clock::realization_requirement(),
            state: Arc::new(ClockRuntimeState::default()),
            lifecycle_capture: config.lifecycle_capture().cloned(),
            realization_capture: config.realization_capture().cloned(),
        }
    }

    fn push_lifecycle(&self, event: &str) {
        if let Some(capture) = &self.lifecycle_capture {
            capture
                .lock()
                .expect("clock lifecycle capture")
                .push(format!("{event}:{}", self.module_id.as_str()));
        }
    }
}

impl ClockRuntimeState {
    fn bind_realization(
        &self,
        resolved: ResolvedContract<ClockRealizationContract>,
    ) -> Arc<ClockRealizationContract> {
        let realization = resolved.into_value();
        *self.realization.lock().expect("clock realization state") = Some(realization.clone());
        realization
    }

    fn current_realization(&self) -> Result<Arc<ClockRealizationContract>, crate::ClockError> {
        self.realization
            .lock()
            .expect("clock realization state")
            .clone()
            .ok_or_else(|| crate::ClockError::Unavailable {
                message: "clock realization is not bound".to_owned(),
            })
    }

    fn clear(&self) -> Result<(), crate::ClockError> {
        if let Some(realization) = self
            .realization
            .lock()
            .expect("clock realization state")
            .clone()
        {
            realization.clear()?;
        }
        Ok(())
    }
}

impl ClockService for RealizedClock {
    fn current_tick(&self) -> Result<ClockTick, crate::ClockError> {
        self.state.current_realization()?.current_tick()
    }
}

impl ModuleRuntime for NativeClock {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![clock_contract_key().declaration()]
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.realization_requirement.declaration().clone()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        let service = Arc::new(RealizedClock {
            state: Arc::clone(&self.state),
        }) as Arc<dyn ClockService>;
        Ok(vec![ModuleContract::new(
            &clock_contract_key(),
            Arc::new(ClockContract::new(service)),
        )])
    }

    fn bind(&mut self, bindings: &ModuleBindings) -> Result<(), ModuleError> {
        let realization = bindings
            .resolve_with_provider(&self.realization_requirement)
            .map_err(|error| ModuleError::new(error.to_string()))?;
        if let Some(capture) = &self.realization_capture {
            *capture.lock().expect("clock realization capture") = Some(BoundClockRealization {
                provider: realization.provider().clone(),
                identity: realization.identity().clone(),
            });
        }
        self.state.bind_realization(realization);
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        self.push_lifecycle("initialize");
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        self.push_lifecycle("start");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        self.push_lifecycle("stop");
        let _ = self.state.clear();
        Ok(())
    }

    fn health(&self) -> Health {
        Health::Healthy
    }
}
