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
pub use registry::{MeshingAttributes, MeshingRegistry};
pub use system::{ChunkCoord, ChunkMesh, MeshDirty};

use bevy::prelude::*;

pub struct VoxelMeshingPlugin;

impl Plugin for VoxelMeshingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<registry::MeshingRegistry>();

        app.add_systems(
            Update,
            (system::dispatch_meshing_tasks, system::apply_meshing_tasks),
        );
    }
}
