use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockState(pub u32);

impl BlockState {
    pub const AIR: BlockState = BlockState(0);
}

pub struct Block {
    pub base_id: u32,
    pub namespace: Arc<str>,
    pub name: Arc<str>,
    pub is_fractional: bool,
    pub state_count: u16,
}
