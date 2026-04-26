use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

use super::{Chunk, ChunkData, attempt_compression};

/// Configuration for the background garbage collector.
#[derive(Resource)]
pub struct ChunkGcConfig {
    /// How many seconds a chunk must be completely untouched before GC considers it.
    pub quiet_cooldown_seconds: f64,
    /// Maximum number of chunks to send to the background thread pool per frame.
    /// Prevents CPU starvation during massive world edits.
    pub max_dispatches_per_frame: usize,
}

impl Default for ChunkGcConfig {
    fn default() -> Self {
        Self {
            quiet_cooldown_seconds: 10.0,
            max_dispatches_per_frame: 10,
        }
    }
}

/// Tracks modifications to determine if a chunk is eligible for downsizing.
#[derive(Component)]
pub struct ChunkActivity {
    pub last_modified: f64,
    pub is_dirty: bool,
}

impl Default for ChunkActivity {
    fn default() -> Self {
        Self {
            last_modified: 0.0,
            is_dirty: false,
        }
    }
}

/// A marker component holding the background compression task.
#[derive(Component)]
pub struct CompressingChunk(Task<Option<ChunkData>>);

/// Dispatches eligible chunks to the background thread pool.
pub fn dispatch_chunk_gc(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<ChunkGcConfig>,
    mut query: Query<(Entity, &Chunk, &mut ChunkActivity), Without<CompressingChunk>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    let current_time = time.elapsed_secs_f64();
    let mut dispatched = 0;

    for (entity, chunk, mut activity) in query.iter_mut() {
        if dispatched >= config.max_dispatches_per_frame {
            break;
        }

        if activity.is_dirty
            && (current_time - activity.last_modified) > config.quiet_cooldown_seconds
        {
            activity.is_dirty = false;

            // Clone the current data array (cheap relative to frame drops)
            let data_clone = chunk.data.clone();

            let task = thread_pool.spawn(async move { attempt_compression(data_clone) });

            commands.entity(entity).insert(CompressingChunk(task));
            dispatched += 1;
        }
    }
}

/// Catches completed tasks from the background threads and applies them to the ECS.
pub fn apply_compressed_chunks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Chunk, &mut CompressingChunk)>,
) {
    for (entity, mut chunk, mut task) in query.iter_mut() {
        if let Some(compression_result) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).remove::<CompressingChunk>();

            if let Some(smaller_data) = compression_result {
                chunk.data = smaller_data;
            }
        }
    }
}
