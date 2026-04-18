//! Block registry is the bridge between human readable block namespace:name ids and optimised
//! engine readable u32 block-states.

// TODO: Helper builder for blocks to also feature an all in one block and item creation system

use crate::voxel::block::{Block, BlockState};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Resource, Default)]
pub struct BlockRegistry {
    /// Safe string lookup for when mods register blocks or logic queries a block by name.
    name_to_block: HashMap<String, Block>,

    /// Flat array for O(1) reverse lookups from BlockState ID -> String.
    /// Uses Arc<str> so multi-state blocks share a single string allocation.
    state_to_name: Vec<Arc<str>>,
}

impl BlockRegistry {
    /// Registers a new block and allocates a contiguous block of state IDs.
    pub fn register(&mut self, namespace: &str, state_count: u32) -> Block {
        if let Some(existing) = self.name_to_block.get(namespace) {
            return Block {
                base_id: existing.base_id,
                state_count: existing.state_count,
                name: Arc::clone(&existing.name),
            };
        }

        let base_id = self.state_to_name.len() as u32;
        let name_arc: Arc<str> = Arc::from(namespace);

        let block = Block {
            base_id,
            state_count,
            name: Arc::clone(&name_arc),
        };

        self.name_to_block.insert(
            namespace.to_string(),
            Block {
                base_id,
                state_count,
                name: Arc::clone(&name_arc),
            },
        );

        for _ in 0..state_count {
            self.state_to_name.push(Arc::clone(&name_arc));
        }

        block
    }

    /// Safe lookup for UI, debugging, and command parsing.
    pub fn get_name(&self, state: BlockState) -> Option<&str> {
        self.state_to_name
            .get(state.0 as usize)
            .map(|arc| arc.as_ref())
    }

    /// Blazing fast, unchecked reverse lookup.
    /// Use this in rendering or hot-path loops.
    #[inline(always)]
    pub fn get_name_unchecked(&self, state: BlockState) -> &str {
        let id = state.0 as usize;

        #[cfg(debug_assertions)]
        {
            &self.state_to_name[id]
        }

        #[cfg(not(debug_assertions))]
        {
            // SAFETY: The caller must guarantee the BlockState was pulled from
            // valid chunk memory, meaning it was registered and exists in the bounds.
            unsafe { self.state_to_name.get_unchecked(id) }
        }
    }
}
