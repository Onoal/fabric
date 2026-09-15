use fabric_core::{
    Block, Composition, CompositionBuilder, CompositionError, CompositionExport, CompositionId,
    ContractProviderSelection,
};

use super::BlockAuthor;
use crate::ids::{IntoBlockId, IntoCompositionId};

pub struct FabricBuilder {
    inner: CompositionBuilder,
}

impl FabricBuilder {
    pub fn new(composition_id: impl IntoCompositionId) -> Result<Self, CompositionError> {
        Ok(Self::from_id(composition_id.into_composition_id()?))
    }

    pub fn from_id(composition_id: CompositionId) -> Self {
        Self {
            inner: CompositionBuilder::new(composition_id),
        }
    }

    pub fn with_block(mut self, block: Block) -> Self {
        self.inner = self.inner.register_block(block);
        self
    }

    pub fn select_provider(mut self, selection: ContractProviderSelection) -> Self {
        self.inner = self.inner.select_provider(selection);
        self
    }

    pub fn export<T>(mut self, export: CompositionExport<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.inner = self.inner.export(export);
        self
    }

    pub fn block(
        self,
        block_id: impl IntoBlockId,
        configure: impl FnOnce(BlockAuthor) -> BlockAuthor,
    ) -> Result<Self, CompositionError> {
        let block = configure(BlockAuthor::new(block_id)?).build();
        Ok(self.with_block(block))
    }

    pub fn build(self) -> Result<Composition, CompositionError> {
        self.inner.build()
    }
}
