#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockState(pub u32);

impl BlockState {
    pub const AIR: BlockState = BlockState(0);
}

pub struct Block {
    pub base_id: u32,
    pub state_count: u32,
    pub name: std::sync::Arc<str>,
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
