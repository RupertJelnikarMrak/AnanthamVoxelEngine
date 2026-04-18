use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::voxel::block::BlockState;
use crate::voxel::chunk::{Chunk, ChunkError};
use crate::voxel::gc::ChunkActivity;

use super::math::{global_to_local, local_to_index};

#[derive(Resource, Default)]
pub struct ChunkMap {
    pub map: HashMap<IVec3, Entity>,
}

#[derive(Debug)]
pub enum WorldAccessError {
    /// The chunk coordinate does not exist in the active ChunkMap.
    ChunkNotLoaded(IVec3),
    /// The ECS entity exists in the map, but it is missing the required Chunk components.
    EcsStateInvalid,
    /// Wraps internal chunk-level errors (like OutOfBounds)
    ChunkError(ChunkError),
}

impl From<ChunkError> for WorldAccessError {
    fn from(err: ChunkError) -> Self {
        WorldAccessError::ChunkError(err)
    }
}

#[derive(SystemParam)]
pub struct WorldVoxelAccessor<'w, 's> {
    chunk_map: Res<'w, ChunkMap>,
    chunk_query: Query<'w, 's, (&'static mut Chunk, &'static mut ChunkActivity)>,
    time: Res<'w, Time>,
}

impl<'w, 's> WorldVoxelAccessor<'w, 's> {
    pub fn get_block(&self, global: IVec3) -> Result<BlockState, WorldAccessError> {
        let (chunk_coord, local_coord) = global_to_local(global);

        let entity = self
            .chunk_map
            .map
            .get(&chunk_coord)
            .ok_or(WorldAccessError::ChunkNotLoaded(chunk_coord))?;

        let (chunk, _) = self
            .chunk_query
            .get(*entity)
            .map_err(|_| WorldAccessError::EcsStateInvalid)?;

        // SAFETY: `global_to_local` uses euclidean remainder, guaranteeing local_coord is exactly
        // 0..31 on all axes.
        unsafe { Ok(chunk.get_block_unchecked(local_to_index(local_coord))) }
    }

    pub fn set_block(&mut self, global: IVec3, state: BlockState) -> Result<(), WorldAccessError> {
        let (chunk_coord, local_coord) = global_to_local(global);

        let entity = self
            .chunk_map
            .map
            .get(&chunk_coord)
            .ok_or(WorldAccessError::ChunkNotLoaded(chunk_coord))?;

        let (mut chunk, mut activity) = self.chunk_query.get_mut(*entity).unwrap();

        // SAFETY: `global_to_local` uses euclidean remainder, guaranteeing local_coord is exactly
        // 0..31 on all axes.
        unsafe {
            let index = local_to_index(local_coord);
            if chunk.get_block_unchecked(index) != state {
                chunk.set_block_unchecked(index, state);
                activity.is_dirty = true;
                activity.last_modified = self.time.elapsed_secs_f64();
            }
        }

        activity.is_dirty = true;
        activity.last_modified = self.time.elapsed_secs_f64();

        Ok(())
    }

    pub fn set_block_batch(&mut self, global_deltas: &[(IVec3, BlockState)]) {
        let mut chunk_groups: HashMap<IVec3, Vec<(usize, BlockState)>> = HashMap::new();

        for (global, state) in global_deltas {
            let (chunk_coord, local_coord) = global_to_local(*global);
            let index = local_to_index(local_coord);
            chunk_groups
                .entry(chunk_coord)
                .or_default()
                .push((index, *state));
        }

        let current_time = self.time.elapsed_secs_f64();

        for (chunk_coord, local_deltas) in chunk_groups {
            if let Some(entity) = self.chunk_map.map.get(&chunk_coord)
                && let Ok((mut chunk, mut activity)) = self.chunk_query.get_mut(*entity)
            {
                // SAFETY: `local_deltas` indices were generated via `local_to_index`
                // mapped from `global_to_local`, mathematically guaranteeing they are within 0..32767.
                unsafe {
                    chunk.set_block_batch_unchecked(&local_deltas);
                }

                activity.is_dirty = true;
                activity.last_modified = current_time;
            }
        }
    }
}
