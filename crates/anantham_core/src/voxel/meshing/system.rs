use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

use crate::voxel::chunk::Chunk;
use crate::voxel::chunk::ChunkActivity;
use crate::voxel::world::ChunkMap;

use super::context::MeshingContext;
use super::greedy::generate_greedy_quads;
use super::meshlet::{Meshlet, build_meshlets};
use super::registry::MeshingRegistry;

/// Marker component added to a chunk when its blocks are modified.
#[derive(Component)]
pub struct MeshDirty;

/// Tracks the spatial coordinate of a chunk entity.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoord(pub IVec3);

/// Holds the async task while it runs in the background.
#[derive(Component)]
pub struct MeshingTask(Task<Vec<Meshlet>>);

/// The final payload attached to the Chunk Entity containing GPU-ready data.
#[derive(Component)]
pub struct ChunkMesh {
    pub meshlets: Vec<Meshlet>,
}

type DirtyChunkFilter = (With<MeshDirty>, Without<MeshingTask>);

pub fn dispatch_meshing_tasks(
    mut commands: Commands,
    registry: Res<MeshingRegistry>,
    chunk_map: Res<ChunkMap>,
    query: Query<(Entity, &Chunk, &ChunkCoord), DirtyChunkFilter>,
    all_chunks: Query<&Chunk>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let registry_arc = (*registry).clone();

    for (entity, center_chunk, coord) in query.iter() {
        let pos = coord.0;

        // [-X, +X, -Y, +Y, -Z, +Z]
        let offsets = [
            IVec3::new(-32, 0, 0),
            IVec3::new(32, 0, 0),
            IVec3::new(0, -32, 0),
            IVec3::new(0, 32, 0),
            IVec3::new(0, 0, -32),
            IVec3::new(0, 0, 32),
        ];

        let mut neighbors: [Option<Chunk>; 6] = Default::default();

        for (i, offset) in offsets.iter().enumerate() {
            let neighbor_pos = pos + *offset;

            if let Some(neighbor_chunk) = chunk_map
                .map
                .get(&neighbor_pos)
                .and_then(|&e| all_chunks.get(e).ok())
            {
                neighbors[i] = Some(neighbor_chunk.clone());
            }
        }

        let context = MeshingContext {
            center: center_chunk.clone(),
            neighbors,
        };

        let reg = registry_arc.clone();
        let task = thread_pool.spawn(async move {
            let raw_quads = generate_greedy_quads(&context, &reg);
            build_meshlets(raw_quads)
        });

        commands
            .entity(entity)
            .remove::<MeshDirty>()
            .insert(MeshingTask(task));
    }
}

pub fn apply_meshing_tasks(mut commands: Commands, mut query: Query<(Entity, &mut MeshingTask)>) {
    for (entity, mut task) in query.iter_mut() {
        if let Some(meshlets) = future::block_on(future::poll_once(&mut task.0)) {
            commands
                .entity(entity)
                .remove::<MeshingTask>()
                .insert(ChunkMesh { meshlets });
        }
    }
}

type CleanChunkFilter = (Without<MeshDirty>, Without<MeshingTask>);

pub fn queue_dirty_chunks_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ChunkActivity), CleanChunkFilter>,
) {
    for (entity, mut activity) in query.iter_mut() {
        if activity.is_dirty {
            commands.entity(entity).insert(MeshDirty);
            activity.is_dirty = false;
        }
    }
}
