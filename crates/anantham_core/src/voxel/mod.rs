pub mod block;
pub mod chunk;
pub mod meshing;
pub mod world;

use bevy::prelude::*;

pub struct VoxelCorePlugin;

impl Plugin for VoxelCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            meshing::VoxelMeshingPlugin,
            block::VoxelBlockPlugin,
            chunk::VoxelChunkPlugin,
            world::VoxelWorldPlugin,
        ));
    }
}
