pub mod context;
pub mod greedy;
pub mod meshlet;
pub mod quad;
pub mod registry;
pub mod system;

pub use context::MeshingContext;
pub use greedy::{CHUNK_SIZE, generate_greedy_quads};
pub use meshlet::{MAX_QUADS_PER_MESHLET, Meshlet, build_meshlets};
pub use quad::{UnpackedQuad, VoxelFace};
pub use registry::{MeshingAssets, MeshingAttributes};
pub use system::{ChunkCoord, ChunkMesh, MeshDirty};

use bevy::prelude::*;

use crate::voxel::block::property::AppBlockPropertyExt;

pub struct VoxelMeshingPlugin;

impl Plugin for VoxelMeshingPlugin {
    fn build(&self, app: &mut App) {
        app.register_block_property::<MeshingAssets>("mesh.ron");

        app.add_systems(
            Update,
            (system::dispatch_meshing_tasks, system::apply_meshing_tasks),
        );
    }
}
