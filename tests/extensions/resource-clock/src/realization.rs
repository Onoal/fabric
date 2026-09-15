use std::sync::Arc;

use fabric_core::{ContractId, ContractIdentity, ContractKey, ContractVersion, ModuleId};

use crate::{ClockError, ClockTick};

const CLOCK_REALIZATION_CONTRACT_ID: &str = "fabric.test.resource.clock.realization";

pub fn clock_realization_contract_id() -> ContractId {
    ContractId::new(CLOCK_REALIZATION_CONTRACT_ID).expect("static clock realization contract id")
}

pub fn clock_realization_contract_key() -> ContractKey<ClockRealizationContract> {
    ContractKey::versioned(
        clock_realization_contract_id(),
        clock_realization_contract_version(),
    )
}

pub fn clock_realization_contract_version() -> ContractVersion {
    ContractVersion::parse("1.0.0").expect("static clock realization contract version")
}

pub trait ClockRealization: Send + Sync + 'static {
    fn current_tick(&self) -> Result<ClockTick, ClockError>;

    fn clear(&self) -> Result<(), ClockError> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundClockRealization {
    pub provider: ModuleId,
    pub identity: ContractIdentity,
}

#[derive(Clone)]
pub struct ClockRealizationContract {
    inner: Arc<dyn ClockRealization>,
}

impl ClockRealizationContract {
    pub fn new(inner: Arc<dyn ClockRealization>) -> Self {
        Self { inner }
    }

    pub fn current_tick(&self) -> Result<ClockTick, ClockError> {
        self.inner.current_tick()
    }

    pub fn clear(&self) -> Result<(), ClockError> {
        self.inner.clear()
    }
}
