use super::property::registry::PropertyPadder;
use crate::voxel::block::{Block, BlockState};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Bridge between the Block and BlockState
/// When a new Block is registered it adds and padds the new block sates to all Property registries.
/// Calling this dirrectly is not recommended. Instead use `BlockBuilder` for non divisible
/// blocks or `MaterialBuilder`for material blocks that need stair, slab, quarter forms.
#[derive(Resource, Default)]
pub struct BlockRegistry {
    name_to_block: HashMap<String, Arc<Block>>,
    state_to_block: Vec<Arc<Block>>,

    property_padders: Vec<Box<dyn PropertyPadder>>,
}

impl BlockRegistry {
    /// ONLY allowed during the Init Stage.
    /// Registers a new property array (like Meshing or Physics) to be auto-padded.
    pub fn register_property_array(&mut self, padder: Box<dyn PropertyPadder>) {
        padder.pad_to(self.state_to_block.len());
        self.property_padders.push(padder);
    }

    /// Allowed at any time (Init or Runtime/Hot-Reloading).
    /// Allocates IDs and instantly synchronizes all registered property arrays.
    pub fn register_block(
        &mut self,
        namespace: &str,
        name: &str,
        display_name: &str,
        state_count: u32,
        is_divisible: bool,
    ) -> Arc<Block> {
        let base_id = self.state_to_block.len() as u32;
        let block = Arc::new(Block {
            base_id,
            state_count,
            namespace: Arc::from(namespace),
            name: Arc::from(name),
            display_name: Arc::from(display_name),
            is_divisible,
        });

        self.name_to_block
            .insert(namespace.to_string(), Arc::clone(&block));

        for _ in 0..state_count {
            self.state_to_block.push(Arc::clone(&block));
        }

        let new_total = self.state_to_block.len();

        for padder in &self.property_padders {
            padder.pad_to(new_total);
        }

        block
    }

    pub fn get_block_by_name(&self, namespace: &str) -> Option<Arc<Block>> {
        self.name_to_block.get(namespace).cloned()
    }

    pub fn get_block(&self, state: BlockState) -> Option<Arc<Block>> {
        self.state_to_block.get(state.0 as usize).cloned()
    }

    /// Blazing fast state-to-block resolution.
    /// # Safety
    /// Caller should guarantee `state` was retrieved from valid, bounds-checked chunk memory.
    #[inline(always)]
    pub unsafe fn get_block_unchecked(&self, state: BlockState) -> Arc<Block> {
        debug_assert!(
            self.state_to_block.len() > state.0 as usize,
            "Attempted to get block unchecked with an unregistered BlockState: {}",
            state.0
        );
        unsafe { Arc::clone(self.state_to_block.get_unchecked(state.0 as usize)) }
    }
}
