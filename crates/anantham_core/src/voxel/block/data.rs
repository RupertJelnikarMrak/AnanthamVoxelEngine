use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockState(pub u32);

impl BlockState {
    pub const AIR: BlockState = BlockState(0);

    #[inline(always)]
    pub fn extract_shape(&self, parent_block: &Block) -> Option<u8> {
        if !parent_block.is_divisible {
            return None;
        }

        let offset = self.0 - parent_block.base_id;

        if offset < 256 {
            Some(offset as u8)
        } else {
            None
        }
    }
}

pub struct Block {
    pub base_id: u32,
    pub state_count: u32,
    pub namespace: Arc<str>,
    pub name: Arc<str>,
    pub display_name: Arc<str>,
    pub is_divisible: bool,
}

impl Block {
    /// Helper to get a specific BlockState from a local property offset
    #[inline(always)]
    pub fn get_state(&self, offset: u32) -> BlockState {
        debug_assert!(
            offset < self.state_count,
            "State offset out of bounds for block"
        );
        BlockState(self.base_id + offset)
    }
}
