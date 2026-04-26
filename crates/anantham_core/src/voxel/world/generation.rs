use bevy::prelude::*;

use crate::voxel::{
    block::{BlockRegistry, BlockState},
    chunk::{Chunk, ChunkActivity},
    meshing::{ChunkCoord, MeshDirty},
    world::{ChunkMap, WorldVoxelAccessor},
};

pub fn spawn_test_world(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    for cx in -2..=2 {
        for cz in -2..=2 {
            let coord = IVec3::new(cx * 32, 0, cz * 32);
            let mut chunk = Chunk::default();

            let test_block = BlockState(
                registry
                    .get_block_by_name("anantham:stone")
                    .map(|b| b.base_id)
                    .unwrap_or(0),
            );

            // let dirt_layer: Vec<(usize, BlockState)> = (0..1024).map(|i| (i, test_block)).collect();
            let mut dirt_layer = Vec::with_capacity(1024);
            for x in 0..32 {
                for y in 0..5 {
                    for z in 0..32 {
                        // Index = x + (y * 32) + (z * 1024). Since y = 0, we omit it.
                        dirt_layer.push(((x + (y * 32) + z * 1024) as usize, test_block));
                    }
                }
            }
            unsafe {
                chunk.set_block_batch_unchecked(&dirt_layer);
            }

            let entity = commands
                .spawn((
                    chunk,
                    ChunkCoord(coord),
                    ChunkActivity::default(),
                    MeshDirty,
                ))
                .id();

            chunk_map.map.insert(coord, entity);
        }
    }
    info!("Test world spawned");
}

pub fn place_test_blocks(mut accessor: WorldVoxelAccessor, registry: Res<BlockRegistry>) {
    let stone_id = registry
        .get_block_by_name("anantham:stone")
        .map(|b| b.base_id)
        .unwrap_or(0);

    let stone = BlockState(stone_id);

    // Place a 3x3 floating stone pillar using absolute world coordinates
    let mut batch = Vec::new();
    for y in 10..15 {
        for x in 0..3 {
            for z in 0..3 {
                batch.push((IVec3::new(x, y, z), stone));
            }
        }
    }

    // The accessor will handle crossing chunk boundaries if necessary
    accessor.set_block_batch(&batch);

    info!("Test blocks spawned");
}
