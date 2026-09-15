use crate::health::Health;
use crate::identifiers::{BlockId, ModuleId};
use crate::lifecycle::LifecycleState;
use crate::module::Module;
use crate::module_runtime::ModuleRuntime;

pub struct BlockBuilder {
    block_id: BlockId,
    modules: Vec<Box<dyn Module>>,
}

pub struct Block {
    pub(crate) block_id: BlockId,
    pub(crate) modules: Vec<Box<dyn Module>>,
}

pub(crate) struct RuntimeBlock {
    pub(crate) block_id: BlockId,
    pub(crate) modules: Vec<Box<dyn ModuleRuntime>>,
    pub(crate) lifecycle: LifecycleState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleReport {
    pub module_id: ModuleId,
    pub health: Health,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockReport {
    pub block_id: BlockId,
    pub lifecycle: LifecycleState,
    pub health: Health,
    pub modules: Vec<ModuleReport>,
}

impl BlockBuilder {
    pub fn new(block_id: BlockId) -> Self {
        Self {
            block_id,
            modules: Vec::new(),
        }
    }

    pub fn register_module(mut self, module: impl Module + 'static) -> Self {
        self.modules.push(Box::new(module));
        self
    }

    pub fn build(self) -> Block {
        Block {
            block_id: self.block_id,
            modules: self.modules,
        }
    }
}

impl Block {
    pub fn id(&self) -> &BlockId {
        &self.block_id
    }
}

impl RuntimeBlock {
    pub fn report(&self) -> BlockReport {
        let modules = self
            .modules
            .iter()
            .map(|module| ModuleReport {
                module_id: module.id().clone(),
                health: module.health(),
            })
            .collect::<Vec<_>>();

        let health = if modules
            .iter()
            .any(|module| module.health == Health::Unavailable)
        {
            Health::Unavailable
        } else if modules
            .iter()
            .any(|module| module.health == Health::Degraded)
        {
            Health::Degraded
        } else {
            Health::Healthy
        };

        BlockReport {
            block_id: self.block_id.clone(),
            lifecycle: self.lifecycle,
            health,
            modules,
        }
    }
}
