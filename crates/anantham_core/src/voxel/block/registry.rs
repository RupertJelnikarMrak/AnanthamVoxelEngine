use crate::voxel::block::{Block, BlockState};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

pub static REGISTERED_STATE_COUNT: AtomicU32 = AtomicU32::new(1);

/// Bridge between the Block and BlockState
/// When a new Block is registered it adds and padds the new block sates to all Property registries.
/// Calling this dirrectly is not recommended. Instead use `BlockBuilder` for non divisible
/// blocks or `MaterialBuilder`for material blocks that need stair, slab, quarter forms.
#[derive(Resource, Default)]
pub struct BlockRegistry {
    name_to_block: HashMap<String, Arc<Block>>,
    state_to_block: Vec<Arc<Block>>,
}

impl BlockRegistry {
    pub fn iter_blocks(&self) -> impl Iterator<Item = &Arc<Block>> {
        self.name_to_block.values()
    }

    pub fn register_block(
        &mut self,
        namespace: &str,
        name: &str,
        is_fractional: bool,
    ) -> Arc<Block> {
        let base_id = self.state_to_block.len() as u32;
        let state_count = if is_fractional { 256 } else { 1 };

        let block = Arc::new(Block {
            base_id,
            namespace: Arc::from(namespace),
            name: Arc::from(name),
            state_count,
            is_fractional,
        });

        let full_name = format!("{}:{}", namespace, name);
        self.name_to_block.insert(full_name, Arc::clone(&block));

        for _ in 0..state_count {
            self.state_to_block.push(Arc::clone(&block));
        }

        let new_total = self.state_to_block.len();

        REGISTERED_STATE_COUNT.store(new_total as u32, Ordering::Release);

        block
    }

    pub fn get_block_by_name(&self, full_name: &str) -> Option<Arc<Block>> {
        self.name_to_block.get(full_name).cloned()
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
