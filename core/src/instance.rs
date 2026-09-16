use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::block::{BlockReport, RuntimeBlock};
use crate::contract::ModuleContract;
use crate::error::InstanceError;
use crate::export::CompositionExport;
use crate::health::Health;
use crate::identifiers::{CompositionId, InstanceId};
use crate::lifecycle::LifecycleState;

static NEXT_INSTANCE_GENERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Process-local identity for one materialized Instance runtime incarnation.
///
/// Fabric mints a fresh value for every materialization. It is not a software
/// version, Composition revision, durable epoch, or globally unique sequence.
pub struct InstanceGeneration(u64);

#[derive(Clone, Debug, PartialEq, Eq)]
/// Generation-scoped runtime context shared by modules in one Instance.
pub struct InstanceRuntimeContext {
    instance_id: InstanceId,
    generation: InstanceGeneration,
}

pub struct Instance {
    composition_id: CompositionId,
    instance_id: InstanceId,
    generation: InstanceGeneration,
    blocks: Vec<RuntimeBlock>,
    start_order: Vec<(usize, usize)>,
    lifecycle: LifecycleState,
    exports: BTreeMap<crate::ContractId, ModuleContract>,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("composition_id", &self.composition_id)
            .field("instance_id", &self.instance_id)
            .field("generation", &self.generation)
            .field("lifecycle", &self.lifecycle)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A bounded current observation of one materialized Instance.
///
/// This report is not an event history, transition journal, or durable
/// lifecycle record. An operation error remains the evidence of a failed
/// operation; this value describes the Instance's current state.
pub struct InstanceReport {
    pub composition_id: CompositionId,
    pub instance_id: InstanceId,
    pub generation: InstanceGeneration,
    pub lifecycle: LifecycleState,
    pub health: Health,
    pub blocks: Vec<BlockReport>,
}

impl InstanceGeneration {
    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub(crate) fn mint() -> Self {
        Self(NEXT_INSTANCE_GENERATION.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for InstanceGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl InstanceRuntimeContext {
    pub(crate) fn new(instance_id: InstanceId, generation: InstanceGeneration) -> Self {
        Self {
            instance_id,
            generation,
        }
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }

    pub fn generation(&self) -> InstanceGeneration {
        self.generation
    }
}

impl Instance {
    pub(crate) fn materialize(
        composition_id: CompositionId,
        context: InstanceRuntimeContext,
        blocks: Vec<RuntimeBlock>,
        start_order: Vec<(usize, usize)>,
        exports: BTreeMap<crate::ContractId, ModuleContract>,
    ) -> Self {
        Self {
            composition_id,
            instance_id: context.instance_id.clone(),
            generation: context.generation,
            blocks,
            start_order,
            lifecycle: LifecycleState::Ready,
            exports,
        }
    }

    pub fn composition_id(&self) -> &CompositionId {
        &self.composition_id
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }

    pub fn generation(&self) -> InstanceGeneration {
        self.generation
    }

    pub fn lifecycle(&self) -> LifecycleState {
        self.lifecycle
    }

    /// Retrieves one explicitly declared Composition export. Arbitrary module
    /// contracts are never retained or available through this API.
    pub fn export<T>(&self, export: &CompositionExport<T>) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let value = self.exports.get(export.id())?;
        if value.declaration().id() != export.requirement().id()
            || !export
                .requirement()
                .compatibility()
                .accepts(value.declaration().identity())
        {
            return None;
        }
        value.value.clone().downcast::<T>().ok()
    }

    pub fn start(&mut self) -> Result<(), InstanceError> {
        if self.lifecycle != LifecycleState::Ready {
            return Err(InstanceError::InvalidLifecycleTransition {
                instance_id: self.instance_id.clone(),
                action: "start",
            });
        }
        let mut initialized = Vec::new();
        let start_order = self.start_order.clone();
        for location in &start_order {
            let module_id = self.blocks[location.0].modules[location.1].id().clone();
            if let Err(source) = self.blocks[location.0].modules[location.1].initialize() {
                self.stop_locations(&initialized);
                self.set_stopped();
                return Err(InstanceError::ModuleFailure {
                    module_id,
                    phase: "initialize",
                    source,
                });
            }
            initialized.push(*location);
        }
        for location in &start_order {
            let module_id = self.blocks[location.0].modules[location.1].id().clone();
            if let Err(source) = self.blocks[location.0].modules[location.1].start() {
                self.stop_locations(&initialized);
                self.set_stopped();
                return Err(InstanceError::ModuleFailure {
                    module_id,
                    phase: "start",
                    source,
                });
            }
        }
        self.lifecycle = LifecycleState::Running;
        for block in &mut self.blocks {
            block.lifecycle = LifecycleState::Running;
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        match self.lifecycle {
            LifecycleState::Ready => self.set_stopped(),
            LifecycleState::Running => {
                let order = self.start_order.clone();
                self.stop_locations(&order);
                self.set_stopped();
            }
            LifecycleState::Stopped => {}
        }
    }

    pub fn report(&self) -> InstanceReport {
        let blocks = self
            .blocks
            .iter()
            .map(RuntimeBlock::report)
            .collect::<Vec<_>>();
        let health = if blocks
            .iter()
            .any(|block| block.health == Health::Unavailable)
        {
            Health::Unavailable
        } else if blocks.iter().any(|block| block.health == Health::Degraded) {
            Health::Degraded
        } else {
            Health::Healthy
        };
        InstanceReport {
            composition_id: self.composition_id.clone(),
            instance_id: self.instance_id.clone(),
            generation: self.generation,
            lifecycle: self.lifecycle,
            health,
            blocks,
        }
    }

    fn stop_locations(&mut self, locations: &[(usize, usize)]) {
        for (block, module) in locations.iter().rev() {
            self.blocks[*block].modules[*module].stop();
        }
    }

    fn set_stopped(&mut self) {
        self.lifecycle = LifecycleState::Stopped;
        for block in &mut self.blocks {
            block.lifecycle = LifecycleState::Stopped;
        }
    }
}
