use crate::voxel::block::BlockState;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoxelFace {
    NegativeX,
    PositiveX,
    NegativeY,
    PositiveY,
    NegativeZ,
    PositiveZ,
}

impl VoxelFace {
    pub const ALL: [VoxelFace; 6] = [
        VoxelFace::NegativeX,
        VoxelFace::PositiveX,
        VoxelFace::NegativeY,
        VoxelFace::PositiveY,
        VoxelFace::NegativeZ,
        VoxelFace::PositiveZ,
    ];
}

#[derive(Debug, Clone)]
pub struct UnpackedQuad {
    pub min: [u8; 3],
    pub width: u8,
    pub height: u8,
    pub face: VoxelFace,
    pub state: BlockState,
    pub material_id: u16,
}
