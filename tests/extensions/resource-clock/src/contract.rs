use std::sync::Arc;

use fabric_core::{ContractId, ContractKey, ContractVersion};

use crate::{ClockError, ClockTick};

const CLOCK_CONTRACT_ID: &str = "fabric.test.resource.clock";

pub fn clock_contract_id() -> ContractId {
    ContractId::new(CLOCK_CONTRACT_ID).expect("static clock contract id")
}

pub fn clock_contract_key() -> ContractKey<ClockContract> {
    ContractKey::versioned(clock_contract_id(), clock_contract_version())
}

pub fn clock_contract_version() -> ContractVersion {
    ContractVersion::parse("1.4.0").expect("static clock contract version")
}

pub trait ClockService: Send + Sync {
    fn current_tick(&self) -> Result<ClockTick, ClockError>;
}

#[derive(Clone)]
pub struct ClockContract {
    inner: Arc<dyn ClockService>,
}

impl ClockContract {
    pub fn new(inner: Arc<dyn ClockService>) -> Self {
        Self { inner }
    }

    pub fn current_tick(&self) -> Result<ClockTick, ClockError> {
        self.inner.current_tick()
    }
}
