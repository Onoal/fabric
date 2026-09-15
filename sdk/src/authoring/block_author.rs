use fabric_core::{Block, BlockBuilder, BlockId, CompositionError, Module};

use crate::ids::IntoBlockId;

pub struct BlockAuthor {
    inner: BlockBuilder,
}

impl BlockAuthor {
    pub fn new(block_id: impl IntoBlockId) -> Result<Self, CompositionError> {
        Ok(Self::from_id(block_id.into_block_id()?))
    }

    pub fn from_id(block_id: BlockId) -> Self {
        Self {
            inner: BlockBuilder::new(block_id),
        }
    }

    pub fn module(mut self, module: impl Module + 'static) -> Self {
        self.inner = self.inner.register_module(module);
        self
    }

    pub fn register_module(self, module: impl Module + 'static) -> Self {
        self.module(module)
    }

    pub fn build(self) -> Block {
        self.inner.build()
    }
}
