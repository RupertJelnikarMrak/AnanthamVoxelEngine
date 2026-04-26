pub mod compression;
pub mod data;
pub mod gc;
pub mod local;

pub use compression::attempt_compression;
pub use data::{CHUNK_SIZE, CHUNK_SIZE_I32, CHUNK_VOLUME, Chunk, ChunkData};
pub use gc::{ChunkActivity, ChunkGcConfig, CompressingChunk};
pub use local::ChunkError;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

pub struct VoxelChunkPlugin;

impl Plugin for VoxelChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<gc::ChunkGcConfig>();

        app.add_systems(
            Update,
            (
                gc::dispatch_chunk_gc.run_if(on_timer(Duration::from_secs(10))),
                gc::apply_compressed_chunks,
            ),
        );
    }
}
