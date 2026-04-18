use crate::voxel::block::data::BlockState;
use crate::voxel::chunk::Chunk;
use bevy::math::IVec3;

/// Provides read-only, owned access to a chunk and its neighbors for async meshing.
pub struct MeshingContext {
    pub center: Chunk,
    // [0: -X, 1: +X, 2: -Y, 3: +Y, 4: -Z, 5: +Z]
    pub neighbors: [Option<Chunk>; 6],
}

impl MeshingContext {
    /// Safely resolves a block request using IVec3, crossing boundaries if needed.
    #[inline(always)]
    pub fn get_block_extended(&self, local: IVec3) -> BlockState {
        // TODO: Currently falls back on AIR, an implementation for error block or something might
        // make more sense
        if local.cmpge(IVec3::ZERO).all() && local.cmplt(IVec3::splat(32)).all() {
            return self.center.get_block(local).unwrap_or(BlockState::AIR);
        }

        let (neighbor_idx, routed_local) = if local.x < 0 {
            (0, IVec3::new(31, local.y, local.z))
        } else if local.x > 31 {
            (1, IVec3::new(0, local.y, local.z))
        } else if local.y < 0 {
            (2, IVec3::new(local.x, 31, local.z))
        } else if local.y > 31 {
            (3, IVec3::new(local.x, 0, local.z))
        } else if local.z < 0 {
            (4, IVec3::new(local.x, local.y, 31))
        } else if local.z > 31 {
            (5, IVec3::new(local.x, local.y, 0))
        } else {
            unreachable!()
        };

        if let Some(neighbor_chunk) = &self.neighbors[neighbor_idx] {
            neighbor_chunk
                .get_block(routed_local)
                .unwrap_or(BlockState::AIR)
        } else {
            BlockState::AIR
        }
    }
}
