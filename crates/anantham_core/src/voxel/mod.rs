pub mod block;
pub mod chunk;
pub mod gc;
pub mod meshing;
pub mod world;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

pub struct VoxelCorePlugin;

impl Plugin for VoxelCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<block::BlockRegistry>();
        app.init_resource::<world::ChunkMap>();
        app.init_resource::<gc::ChunkGcConfig>();

        app.add_plugins(meshing::VoxelMeshingPlugin);

        app.add_systems(
            Update,
            (
                gc::dispatch_chunk_gc.run_if(on_timer(Duration::from_secs(10))),
                gc::apply_compressed_chunks,
            ),
        );
    }
}
